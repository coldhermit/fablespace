//! Field menu and the save screen.

use fablespace_data::{ArmorSlot, Direction, Occasion, Scope};

use crate::battle::field_recover;
use crate::interpret::WaitKind;
use crate::party::{self, Member};
use crate::persist::{self, SavePreview, SLOT_COUNT};
use crate::session::{Input, Screen, Session};

#[derive(Clone, Debug)]
pub enum MenuPage {
    Root,
    Item {
        cursor: usize,
        actor: usize,
    },
    Skill {
        cursor: usize,
        actor: usize,
    },
    Equip {
        slot: usize,
        actor: usize,
    },
    EquipPick {
        slot: usize,
        actor: usize,
        cursor: usize,
    },
    Status {
        actor: usize,
    },
    End {
        yes: bool,
    },
}

pub struct Menu {
    pub cursor: usize,
    pub page: MenuPage,
}

pub struct SaveUi {
    pub loading: bool,
    pub cursor: usize,
    pub confirm: bool,
    pub slots: [Option<SavePreview>; SLOT_COUNT],
}

pub const EQUIP_LABELS: [&str; 5] = ["Weapon", "Shield", "Head", "Body", "Accessory"];

impl Session {
    pub(crate) fn open_menu(&mut self) {
        if self.menu.is_some() || self.party.is_empty() {
            return;
        }
        self.menu = Some(Menu {
            cursor: 0,
            page: MenuPage::Root,
        });
    }

    pub(crate) fn open_save(&mut self, loading: bool) {
        let mut slots = [None, None, None, None];
        for (index, slot) in slots.iter_mut().enumerate() {
            *slot = self.slot_preview(index);
        }
        self.save_ui = Some(SaveUi {
            loading,
            cursor: 0,
            confirm: false,
            slots,
        });
    }

    pub(crate) fn update_menu(&mut self, input: &Input) {
        let Some(menu) = self.menu.clone_page() else {
            return;
        };
        match menu {
            MenuPage::Root => self.menu_root(input),
            MenuPage::Item { cursor, actor } => self.menu_item(input, cursor, actor),
            MenuPage::Skill { cursor, actor } => self.menu_skill(input, cursor, actor),
            MenuPage::Equip { slot, actor } => self.menu_equip(input, slot, actor),
            MenuPage::EquipPick {
                slot,
                actor,
                cursor,
            } => self.menu_equip_pick(input, slot, actor, cursor),
            MenuPage::Status { actor } => self.menu_status(input, actor),
            MenuPage::End { yes } => self.menu_end(input, yes),
        }
    }

    pub(crate) fn update_save(&mut self, input: &Input) {
        let Some(ui) = self.save_ui.as_mut() else {
            return;
        };
        let loading = ui.loading;
        if input.dir == Some(Direction::Up) {
            ui.cursor = (ui.cursor + SLOT_COUNT - 1) % SLOT_COUNT;
            ui.confirm = false;
        }
        if input.dir == Some(Direction::Down) {
            ui.cursor = (ui.cursor + 1) % SLOT_COUNT;
            ui.confirm = false;
        }
        if input.cancel {
            if ui.confirm {
                ui.confirm = false;
                return;
            }
            self.close_save();
            return;
        }
        if !input.confirm {
            return;
        }
        let cursor = self.save_ui.as_ref().map(|ui| ui.cursor).unwrap_or(0);
        let occupied = self
            .save_ui
            .as_ref()
            .is_some_and(|ui| ui.slots[cursor].is_some());
        let confirming = self.save_ui.as_ref().is_some_and(|ui| ui.confirm);
        if loading {
            if !occupied {
                self.notify("This slot is empty.");
                return;
            }
            match self.load_slot(cursor) {
                Ok(()) => self.notify("Game loaded."),
                Err(err) => self.notify(err),
            }
            return;
        }
        if occupied && !confirming {
            if let Some(ui) = &mut self.save_ui {
                ui.confirm = true;
            }
            return;
        }
        match self.save_slot(cursor) {
            Ok(()) => {
                self.notify("Game saved.");
                self.refresh_save_slots();
            }
            Err(err) => self.notify(err),
        }
    }

    fn menu_root(&mut self, input: &Input) {
        let len = 6;
        if input.dir == Some(Direction::Up) {
            if let Some(menu) = &mut self.menu {
                menu.cursor = (menu.cursor + len - 1) % len;
            }
        }
        if input.dir == Some(Direction::Down) {
            if let Some(menu) = &mut self.menu {
                menu.cursor = (menu.cursor + 1) % len;
            }
        }
        if input.cancel {
            self.close_menu();
            return;
        }
        if !input.confirm {
            return;
        }
        let cursor = self.menu.as_ref().map(|menu| menu.cursor).unwrap_or(0);
        let page = match cursor {
            0 => MenuPage::Item {
                cursor: 0,
                actor: 0,
            },
            1 => MenuPage::Skill {
                cursor: 0,
                actor: 0,
            },
            2 => MenuPage::Equip { slot: 0, actor: 0 },
            3 => MenuPage::Status { actor: 0 },
            4 => {
                self.open_save(false);
                return;
            }
            _ => MenuPage::End { yes: false },
        };
        if let Some(menu) = &mut self.menu {
            menu.page = page;
        }
    }

    fn menu_item(&mut self, input: &Input, mut cursor: usize, mut actor: usize) {
        let items = self.menu_items();
        if items.is_empty() {
            if input.cancel || input.confirm {
                self.back_to_root();
            }
            return;
        }
        cursor = cursor.min(items.len() - 1);
        actor = actor.min(self.party.len().saturating_sub(1));
        self.nudge_list(input, &mut cursor, items.len(), &mut actor);
        if let Some(menu) = &mut self.menu {
            menu.page = MenuPage::Item { cursor, actor };
        }
        if input.cancel {
            self.back_to_root();
            return;
        }
        if input.confirm {
            let id = items[cursor];
            self.use_field_item(id, actor);
        }
    }

    fn menu_skill(&mut self, input: &Input, mut cursor: usize, mut actor: usize) {
        actor = actor.min(self.party.len().saturating_sub(1));
        let skills = self.menu_skills(actor);
        if skills.is_empty() {
            if input.dir == Some(Direction::Left) || input.dir == Some(Direction::Right) {
                self.cycle_actor(input, &mut actor);
                if let Some(menu) = &mut self.menu {
                    menu.page = MenuPage::Skill { cursor: 0, actor };
                }
                return;
            }
            if input.cancel || input.confirm {
                self.back_to_root();
            }
            return;
        }
        cursor = cursor.min(skills.len() - 1);
        self.nudge_list(input, &mut cursor, skills.len(), &mut actor);
        if let Some(menu) = &mut self.menu {
            menu.page = MenuPage::Skill { cursor, actor };
        }
        if input.cancel {
            self.back_to_root();
            return;
        }
        if input.confirm {
            let id = skills[cursor];
            self.use_field_skill(id, actor);
        }
    }

    fn menu_equip(&mut self, input: &Input, mut slot: usize, mut actor: usize) {
        actor = actor.min(self.party.len().saturating_sub(1));
        if input.dir == Some(Direction::Up) {
            slot = (slot + EQUIP_LABELS.len() - 1) % EQUIP_LABELS.len();
        }
        if input.dir == Some(Direction::Down) {
            slot = (slot + 1) % EQUIP_LABELS.len();
        }
        self.cycle_actor(input, &mut actor);
        if let Some(menu) = &mut self.menu {
            menu.page = MenuPage::Equip { slot, actor };
        }
        if input.cancel {
            self.back_to_root();
            return;
        }
        if input.confirm {
            if let Some(menu) = &mut self.menu {
                menu.page = MenuPage::EquipPick {
                    slot,
                    actor,
                    cursor: 0,
                };
            }
        }
    }

    fn menu_equip_pick(&mut self, input: &Input, slot: usize, actor: usize, mut cursor: usize) {
        let choices = self.equip_choices(slot);
        if choices.is_empty() {
            if input.cancel || input.confirm {
                if let Some(menu) = &mut self.menu {
                    menu.page = MenuPage::Equip { slot, actor };
                }
            }
            return;
        }
        let len = choices.len();
        if input.dir == Some(Direction::Up) {
            cursor = (cursor + len - 1) % len;
        }
        if input.dir == Some(Direction::Down) {
            cursor = (cursor + 1) % len;
        }
        if let Some(menu) = &mut self.menu {
            menu.page = MenuPage::EquipPick {
                slot,
                actor,
                cursor,
            };
        }
        if input.cancel {
            if let Some(menu) = &mut self.menu {
                menu.page = MenuPage::Equip { slot, actor };
            }
            return;
        }
        if input.confirm {
            self.apply_equip(actor, slot, choices[cursor]);
            if let Some(menu) = &mut self.menu {
                menu.page = MenuPage::Equip { slot, actor };
            }
        }
    }

    fn menu_status(&mut self, input: &Input, mut actor: usize) {
        actor = actor.min(self.party.len().saturating_sub(1));
        self.cycle_actor(input, &mut actor);
        if let Some(menu) = &mut self.menu {
            menu.page = MenuPage::Status { actor };
        }
        if input.cancel || input.confirm {
            self.back_to_root();
        }
    }

    fn menu_end(&mut self, input: &Input, mut yes: bool) {
        if input.dir == Some(Direction::Left) || input.dir == Some(Direction::Up) {
            yes = false;
        }
        if input.dir == Some(Direction::Right) || input.dir == Some(Direction::Down) {
            yes = true;
        }
        if let Some(menu) = &mut self.menu {
            menu.page = MenuPage::End { yes };
        }
        if input.cancel {
            self.back_to_root();
            return;
        }
        if input.confirm && yes {
            self.to_title();
        } else if input.confirm {
            self.back_to_root();
        }
    }

    fn use_field_item(&mut self, id: u32, actor: usize) {
        let Some(item) = self.game.item(id).cloned() else {
            return;
        };
        if !item.occasion.allows_menu() {
            return;
        }
        let targets = self.field_targets(item.scope, actor);
        if item.consumable && !self.bag.lose_item(id, 1) {
            self.notify("No items left.");
            return;
        }
        let mut lines = vec![format!("Used {}.", item.name)];
        for index in targets {
            lines.extend(field_recover(self, index, &item.effects, &item.damage));
        }
        self.show_lines(String::new(), String::new(), lines);
    }

    fn use_field_skill(&mut self, id: u32, actor: usize) {
        let Some(skill) = self.game.skill(id).cloned() else {
            return;
        };
        if !skill.occasion.allows_menu() || actor >= self.party.len() {
            return;
        }
        if self.party[actor].mp < skill.mp_cost {
            self.notify("Not enough MP.");
            return;
        }
        self.party[actor].mp -= skill.mp_cost;
        let mut lines = vec![format!("Used {}.", skill.name)];
        for index in self.field_targets(skill.scope, actor) {
            lines.extend(field_recover(self, index, &skill.effects, &skill.damage));
        }
        self.show_lines(String::new(), String::new(), lines);
    }

    fn field_targets(&self, scope: Scope, actor: usize) -> Vec<usize> {
        match scope {
            Scope::AllAllies | Scope::AllEnemies => (0..self.party.len()).collect(),
            _ => vec![actor.min(self.party.len().saturating_sub(1))],
        }
    }

    fn apply_equip(&mut self, actor: usize, slot: usize, choice: EquipChoice) {
        if actor >= self.party.len() {
            return;
        }
        match choice {
            EquipChoice::Remove => self.unequip(actor, slot),
            EquipChoice::Weapon(id) => {
                self.unequip(actor, slot);
                if self.bag.lose_weapon(id, 1) {
                    self.party[actor].weapon_id = id;
                }
            }
            EquipChoice::Armor(id) => {
                self.unequip(actor, slot);
                if self.bag.lose_armor(id, 1) {
                    self.set_armor(actor, slot, id);
                }
            }
        }
        self.clamp_vitals(actor);
    }

    fn unequip(&mut self, actor: usize, slot: usize) {
        if slot == 0 {
            let id = self.party[actor].weapon_id;
            if id != 0 {
                self.bag.gain_weapon(id, 1);
                self.party[actor].weapon_id = 0;
            }
            return;
        }
        let id = self.armor_id(&self.party[actor], slot);
        if id != 0 {
            self.bag.gain_armor(id, 1);
            self.set_armor(actor, slot, 0);
        }
    }

    fn set_armor(&mut self, actor: usize, slot: usize, id: u32) {
        let member = &mut self.party[actor];
        match slot {
            1 => member.shield_id = id,
            2 => member.head_id = id,
            3 => member.body_id = id,
            _ => member.accessory_id = id,
        }
    }

    fn armor_id(&self, member: &Member, slot: usize) -> u32 {
        match slot {
            1 => member.shield_id,
            2 => member.head_id,
            3 => member.body_id,
            _ => member.accessory_id,
        }
    }

    fn clamp_vitals(&mut self, actor: usize) {
        let stats = party::stats_of(&self.game, &self.party[actor]);
        self.party[actor].hp = self.party[actor].hp.min(stats.mhp).max(0);
        self.party[actor].mp = self.party[actor].mp.min(stats.mmp).max(0);
    }

    pub(crate) fn equip_choices(&self, slot: usize) -> Vec<EquipChoice> {
        let mut choices = vec![EquipChoice::Remove];
        if slot == 0 {
            for id in self.bag.weapons.keys().copied() {
                choices.push(EquipChoice::Weapon(id));
            }
        } else {
            let wanted = match slot {
                1 => ArmorSlot::Shield,
                2 => ArmorSlot::Head,
                4 => ArmorSlot::Accessory,
                _ => ArmorSlot::Body,
            };
            for id in self.bag.armors.keys().copied() {
                if self
                    .game
                    .armor(id)
                    .is_some_and(|armor| armor.slot == wanted)
                {
                    choices.push(EquipChoice::Armor(id));
                }
            }
        }
        choices
    }

    pub(crate) fn equipped_name(&self, actor: usize, slot: usize) -> String {
        let Some(member) = self.party.get(actor) else {
            return "—".into();
        };
        if slot == 0 {
            return self
                .game
                .weapon(member.weapon_id)
                .map(|weapon| weapon.name.clone())
                .unwrap_or_else(|| "—".into());
        }
        self.game
            .armor(self.armor_id(member, slot))
            .map(|armor| armor.name.clone())
            .unwrap_or_else(|| "—".into())
    }

    pub(crate) fn menu_items(&self) -> Vec<u32> {
        self.bag
            .items
            .iter()
            .filter(|(id, count)| {
                **count > 0
                    && self
                        .game
                        .item(**id)
                        .is_some_and(|item| item.occasion.allows_menu())
            })
            .map(|(id, _)| *id)
            .collect()
    }

    pub(crate) fn menu_skills(&self, actor: usize) -> Vec<u32> {
        let Some(member) = self.party.get(actor) else {
            return Vec::new();
        };
        party::known_skills(&self.game, member)
            .into_iter()
            .filter(|id| {
                self.game
                    .skill(*id)
                    .is_some_and(|skill| skill.occasion.allows_menu())
            })
            .collect()
    }

    fn nudge_list(&self, input: &Input, cursor: &mut usize, len: usize, actor: &mut usize) {
        if input.dir == Some(Direction::Up) && len > 0 {
            *cursor = (*cursor + len - 1) % len;
        }
        if input.dir == Some(Direction::Down) && len > 0 {
            *cursor = (*cursor + 1) % len;
        }
        self.cycle_actor(input, actor);
    }

    fn cycle_actor(&self, input: &Input, actor: &mut usize) {
        let len = self.party.len().max(1);
        if input.dir == Some(Direction::Left) {
            *actor = (*actor + len - 1) % len;
        }
        if input.dir == Some(Direction::Right) {
            *actor = (*actor + 1) % len;
        }
    }

    fn back_to_root(&mut self) {
        if let Some(menu) = &mut self.menu {
            menu.page = MenuPage::Root;
        }
    }

    fn close_menu(&mut self) {
        self.menu = None;
        self.clear_wait(WaitKind::Menu);
        if self.screen == Screen::Map && self.save_ui.is_none() && self.battle.is_none() {
            self.drive_foreground();
        }
    }

    fn close_save(&mut self) {
        self.save_ui = None;
        if self.screen != Screen::Map {
            return;
        }
        self.clear_wait(WaitKind::Save);
        if self.menu.is_none() && self.battle.is_none() {
            self.drive_foreground();
        }
    }

    fn refresh_save_slots(&mut self) {
        let previews = [0, 1, 2, 3].map(|slot| self.slot_preview(slot));
        if let Some(ui) = &mut self.save_ui {
            ui.slots = previews;
            ui.confirm = false;
        }
    }

    pub fn actor_label(&self, index: usize) -> String {
        self.party
            .get(index)
            .and_then(|member| self.game.actor(member.actor_id))
            .map(|actor| actor.name.clone())
            .unwrap_or_else(|| format!("Actor {}", index + 1))
    }
}

trait MenuPageCopy {
    fn clone_page(&self) -> Option<MenuPage>;
}

impl MenuPageCopy for Option<Menu> {
    fn clone_page(&self) -> Option<MenuPage> {
        self.as_ref().map(|menu| menu.page.clone())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EquipChoice {
    Remove,
    Weapon(u32),
    Armor(u32),
}

impl EquipChoice {
    pub fn label(self, session: &Session) -> String {
        match self {
            EquipChoice::Remove => "Remove".into(),
            EquipChoice::Weapon(id) => session
                .game
                .weapon(id)
                .map(|weapon| weapon.name.clone())
                .unwrap_or_else(|| format!("Weapon {id}")),
            EquipChoice::Armor(id) => session
                .game
                .armor(id)
                .map(|armor| armor.name.clone())
                .unwrap_or_else(|| format!("Armor {id}")),
        }
    }
}

pub fn play_clock(frames: u64) -> String {
    persist::format_play(frames)
}

#[allow(dead_code)]
fn _occasion_menu(occasion: Occasion) -> bool {
    occasion.allows_menu()
}
