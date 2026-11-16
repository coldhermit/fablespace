//! Database forms and the event command editor.

use fablespace_data::command::{self, Command, MoveRoute, MoveStep, Slot, Step, Target};
use fablespace_data::*;
use iced::widget::{button, checkbox, column, pick_list, row, text, text_input, Column};
use iced::{Element, Length};

use super::chrome;
use super::{Maker, Message, Screen};

impl Maker {
    pub(crate) fn apply_text(&mut self, key: &str, value: String) {
        if let Some(index) = key
            .strip_prefix("switch.")
            .and_then(|rest| rest.parse::<usize>().ok())
        {
            if index > 0 && index < self.game.switches.len() {
                self.game.switches[index] = value;
                self.changed();
            }
            return;
        }
        if let Some(index) = key
            .strip_prefix("variable.")
            .and_then(|rest| rest.parse::<usize>().ok())
        {
            if index > 0 && index < self.game.variables.len() {
                self.game.variables[index] = value;
                self.changed();
            }
            return;
        }
        if let Some(index) = key
            .strip_prefix("line.")
            .and_then(|rest| rest.parse::<usize>().ok())
        {
            self.update_cmd(|cmd| {
                if let Command::ShowText { lines, .. } = cmd {
                    lines.resize(4, String::new());
                    if let Some(line) = lines.get_mut(index) {
                        *line = value.clone();
                        return true;
                    }
                }
                false
            });
            return;
        }
        if let Some(index) = key
            .strip_prefix("choice.")
            .and_then(|rest| rest.parse::<usize>().ok())
        {
            self.update_cmd(|cmd| {
                if let Command::ShowChoices { choices, .. } = cmd {
                    if let Some(choice) = choices.get_mut(index) {
                        *choice = value.clone();
                        return true;
                    }
                }
                false
            });
            return;
        }
        if key == "system.party" {
            self.game.system.party = value
                .split(|c: char| !c.is_ascii_digit())
                .filter(|part| !part.is_empty())
                .filter_map(|part| part.parse::<u32>().ok())
                .filter(|id| *id != 0)
                .collect();
            self.changed();
            return;
        }
        let wrote = match key {
            "map.name" => self.map_mut().map(|map| map.name = value.clone()).is_some(),
            "actor.name" => self
                .actor_mut()
                .map(|actor| actor.name = value.clone())
                .is_some(),
            "actor.nickname" => self
                .actor_mut()
                .map(|actor| actor.nickname = value.clone())
                .is_some(),
            "actor.profile" => self
                .actor_mut()
                .map(|actor| actor.profile = value.clone())
                .is_some(),
            "actor.face" => self
                .actor_mut()
                .map(|actor| actor.face = value.clone())
                .is_some(),
            "actor.character" => self
                .actor_mut()
                .map(|actor| actor.character = value.clone())
                .is_some(),
            "actor.note" => self
                .actor_mut()
                .map(|actor| actor.note = value.clone())
                .is_some(),
            "class.name" => self
                .class_mut()
                .map(|class| class.name = value.clone())
                .is_some(),
            "class.note" => self
                .class_mut()
                .map(|class| class.note = value.clone())
                .is_some(),
            "skill.name" => self
                .skill_mut()
                .map(|skill| skill.name = value.clone())
                .is_some(),
            "skill.description" => self
                .skill_mut()
                .map(|skill| skill.description = value.clone())
                .is_some(),
            "skill.note" => self
                .skill_mut()
                .map(|skill| skill.note = value.clone())
                .is_some(),
            "item.name" => self
                .item_mut()
                .map(|item| item.name = value.clone())
                .is_some(),
            "item.description" => self
                .item_mut()
                .map(|item| item.description = value.clone())
                .is_some(),
            "item.note" => self
                .item_mut()
                .map(|item| item.note = value.clone())
                .is_some(),
            "weapon.name" => self
                .weapon_mut()
                .map(|weapon| weapon.name = value.clone())
                .is_some(),
            "weapon.description" => self
                .weapon_mut()
                .map(|weapon| weapon.description = value.clone())
                .is_some(),
            "weapon.note" => self
                .weapon_mut()
                .map(|weapon| weapon.note = value.clone())
                .is_some(),
            "armor.name" => self
                .armor_mut()
                .map(|armor| armor.name = value.clone())
                .is_some(),
            "armor.description" => self
                .armor_mut()
                .map(|armor| armor.description = value.clone())
                .is_some(),
            "armor.note" => self
                .armor_mut()
                .map(|armor| armor.note = value.clone())
                .is_some(),
            "enemy.name" => self
                .enemy_mut()
                .map(|enemy| enemy.name = value.clone())
                .is_some(),
            "enemy.battler" => self
                .enemy_mut()
                .map(|enemy| enemy.battler = value.clone())
                .is_some(),
            "enemy.note" => self
                .enemy_mut()
                .map(|enemy| enemy.note = value.clone())
                .is_some(),
            "troop.name" => self
                .troop_mut()
                .map(|troop| troop.name = value.clone())
                .is_some(),
            "state.name" => self
                .state_mut()
                .map(|state| state.name = value.clone())
                .is_some(),
            "state.note" => self
                .state_mut()
                .map(|state| state.note = value.clone())
                .is_some(),
            "tileset.name" => self
                .tileset_mut()
                .map(|tileset| tileset.name = value.clone())
                .is_some(),
            "tileset.image" => self
                .tileset_mut()
                .map(|tileset| tileset.image = value.clone())
                .is_some(),
            "tileset.note" => self
                .tileset_mut()
                .map(|tileset| tileset.note = value.clone())
                .is_some(),
            "common.name" => self
                .common_mut()
                .map(|event| event.name = value.clone())
                .is_some(),
            "system.title" => {
                self.game.system.game_title = value.clone();
                true
            }
            "system.currency" => {
                self.game.system.currency = value.clone();
                true
            }
            "system.title_image" => {
                self.game.system.title_image = value.clone();
                true
            }
            "event.name" => self
                .event_mut()
                .map(|event| event.name = value.clone())
                .is_some(),
            "page.character" => self
                .page_mut()
                .map(|page| page.image.character = value.clone())
                .is_some(),
            "comment" => {
                self.update_cmd(|cmd| {
                    if let Command::Comment { text } = cmd {
                        *text = value.clone();
                        true
                    } else {
                        false
                    }
                });
                return;
            }
            "picture.name" => {
                self.update_cmd(|cmd| {
                    if let Command::ShowPicture { name, .. } = cmd {
                        *name = value.clone();
                        true
                    } else {
                        false
                    }
                });
                return;
            }
            "face" => {
                self.update_cmd(|cmd| {
                    if let Command::ShowText { face, .. } = cmd {
                        *face = value.clone();
                        true
                    } else {
                        false
                    }
                });
                return;
            }
            "speaker" => {
                self.update_cmd(|cmd| {
                    if let Command::ShowText { speaker, .. } = cmd {
                        *speaker = value.clone();
                        true
                    } else {
                        false
                    }
                });
                return;
            }
            _ => false,
        };
        if let Some(field) = key.strip_prefix("system.term.") {
            wrote_term(&mut self.game.system.terms, field, value);
            self.changed();
            return;
        }
        if wrote {
            self.changed();
        }
    }

    pub(crate) fn apply_num(&mut self, key: &str, value: String) {
        self.num_key = key.to_string();
        self.num_buf = value;
        if self.num_buf.is_empty() || self.num_buf == "-" {
            return;
        }
        if self.write_prefixed_num(key) {
            self.changed();
        }
    }

    fn write_prefixed_num(&mut self, key: &str) -> bool {
        if let Some(rest) = key.strip_prefix("class.base.") {
            return self.set_growth(rest, true);
        }
        if let Some(rest) = key.strip_prefix("class.plus.") {
            return self.set_growth(rest, false);
        }
        if let Some(rest) = key.strip_prefix("weapon.param.") {
            return self.set_weapon_param(rest);
        }
        if let Some(rest) = key.strip_prefix("armor.param.") {
            return self.set_armor_param(rest);
        }
        if let Some(rest) = key.strip_prefix("enemy.param.") {
            return self.set_enemy_param(rest);
        }
        if let Some(rest) = key.strip_prefix("stack.") {
            return self.set_stack_num(rest);
        }
        if let Some(rest) = key.strip_prefix("effect.") {
            return self.set_effect_num(rest);
        }
        if let Some(rest) = key.strip_prefix("learning.") {
            return self.set_learning_num(rest);
        }
        if let Some(rest) = key.strip_prefix("drop.") {
            return self.set_drop_num(rest);
        }
        if let Some(rest) = key.strip_prefix("action.") {
            return self.set_action_num(rest);
        }
        if let Some(rest) = key.strip_prefix("member.") {
            return self.set_member_num(rest);
        }
        if let Some(rest) = key.strip_prefix("auto.") {
            return self.set_autotile_num(rest);
        }
        if let Some(rest) = key.strip_prefix("route.") {
            return self.set_route_num(rest);
        }
        self.write_plain_num(key)
    }

    fn write_plain_num(&mut self, key: &str) -> bool {
        let Some(signed) = self.num_buf.parse::<i32>().ok() else {
            return false;
        };
        let unsigned = self.num_buf.parse::<u32>().ok();
        match key {
            "class.basis" => self
                .class_mut()
                .map(|class| class.exp.basis = signed.max(0))
                .is_some(),
            "class.extra" => self
                .class_mut()
                .map(|class| class.exp.extra = signed.max(0))
                .is_some(),
            "class.accel" => self
                .class_mut()
                .map(|class| class.exp.accel = signed.max(0))
                .is_some(),
            "skill.mp" => self
                .skill_mut()
                .map(|skill| skill.mp_cost = signed.max(0))
                .is_some(),
            "skill.hit" => self
                .skill_mut()
                .map(|skill| skill.hit = signed.clamp(0, 100))
                .is_some(),
            "skill.power" => self
                .skill_mut()
                .map(|skill| skill.damage.power = signed)
                .is_some(),
            "skill.variance" => self
                .skill_mut()
                .map(|skill| skill.damage.variance = signed.clamp(0, 100))
                .is_some(),
            "item.price" => self
                .item_mut()
                .map(|item| item.price = signed.max(0))
                .is_some(),
            "item.hit" => self
                .item_mut()
                .map(|item| item.hit = signed.clamp(0, 100))
                .is_some(),
            "item.power" => self
                .item_mut()
                .map(|item| item.damage.power = signed)
                .is_some(),
            "item.variance" => self
                .item_mut()
                .map(|item| item.damage.variance = signed.clamp(0, 100))
                .is_some(),
            "weapon.price" => self
                .weapon_mut()
                .map(|weapon| weapon.price = signed.max(0))
                .is_some(),
            "armor.price" => self
                .armor_mut()
                .map(|armor| armor.price = signed.max(0))
                .is_some(),
            "enemy.exp" => self
                .enemy_mut()
                .map(|enemy| enemy.exp = signed.max(0))
                .is_some(),
            "enemy.gold" => self
                .enemy_mut()
                .map(|enemy| enemy.gold = signed.max(0))
                .is_some(),
            "state.slip" => self
                .state_mut()
                .map(|state| state.slip_hp_percent = signed.clamp(0, 100))
                .is_some(),
            "state.turn_min" => self.set_state_turns(true, unsigned.unwrap_or(0)),
            "state.turn_max" => self.set_state_turns(false, unsigned.unwrap_or(0)),
            "tileset.columns" => self
                .tileset_mut()
                .map(|tileset| tileset.columns = unsigned.unwrap_or(1).clamp(1, 64))
                .is_some(),
            "common.switch" => self
                .common_mut()
                .map(|event| event.switch_id = unsigned.unwrap_or(0))
                .is_some(),
            "system.start_map" => {
                self.game.system.start_map_id = unsigned.unwrap_or(1);
                true
            }
            "system.start_x" => {
                self.game.system.start_x = unsigned.unwrap_or(0);
                true
            }
            "system.start_y" => {
                self.game.system.start_y = unsigned.unwrap_or(0);
                true
            }
            "system.start_gold" => {
                self.game.system.start_gold = signed.max(0);
                true
            }
            "actor.level" => self
                .actor_mut()
                .map(|actor| actor.initial_level = unsigned.unwrap_or(1).max(1))
                .is_some(),
            "actor.max_level" => self
                .actor_mut()
                .map(|actor| actor.max_level = unsigned.unwrap_or(1).max(1))
                .is_some(),
            "actor.char_index" => self
                .actor_mut()
                .map(|actor| actor.character_index = unsigned.unwrap_or(0))
                .is_some(),
            "event.x" => self
                .event_mut()
                .map(|event| event.x = unsigned.unwrap_or(0))
                .is_some(),
            "event.y" => self
                .event_mut()
                .map(|event| event.y = unsigned.unwrap_or(0))
                .is_some(),
            "page.sw1" => self
                .page_mut()
                .map(|page| page.conditions.switch_1 = unsigned.unwrap_or(0))
                .is_some(),
            "page.sw2" => self
                .page_mut()
                .map(|page| page.conditions.switch_2 = unsigned.unwrap_or(0))
                .is_some(),
            "page.var" => self
                .page_mut()
                .map(|page| page.conditions.variable_id = unsigned.unwrap_or(0))
                .is_some(),
            "page.varmin" => self
                .page_mut()
                .map(|page| page.conditions.variable_min = signed)
                .is_some(),
            "page.item" => self
                .page_mut()
                .map(|page| page.conditions.item_id = unsigned.unwrap_or(0))
                .is_some(),
            "page.actor" => self
                .page_mut()
                .map(|page| page.conditions.actor_id = unsigned.unwrap_or(0))
                .is_some(),
            "page.index" => self
                .page_mut()
                .map(|page| page.image.index = unsigned.unwrap_or(0))
                .is_some(),
            "page.pattern" => self
                .page_mut()
                .map(|page| page.image.pattern = unsigned.unwrap_or(1).clamp(0, 2) as u8)
                .is_some(),
            "page.speed" => self
                .page_mut()
                .map(|page| page.move_speed = unsigned.unwrap_or(4).clamp(1, 5) as u8)
                .is_some(),
            "page.frequency" => self
                .page_mut()
                .map(|page| page.move_frequency = unsigned.unwrap_or(3).clamp(1, 5) as u8)
                .is_some(),
            "troop.turn" => self
                .troop_page_mut()
                .map(|page| page.condition.turn_start = unsigned.filter(|n| *n > 0))
                .is_some(),
            "troop.every" => self
                .troop_page_mut()
                .map(|page| page.condition.turn_every = unsigned.unwrap_or(0))
                .is_some(),
            "troop.hp" => self.set_troop_hp(signed),
            "troop.switch" => self
                .troop_page_mut()
                .map(|page| page.condition.switch_id = unsigned.unwrap_or(0))
                .is_some(),
            "troop.member" => self.set_troop_member(unsigned.unwrap_or(0) as usize),
            "cmd.switch" => self.set_cmd_u32("switch", unsigned.unwrap_or(1)),
            "cmd.var" => self.set_cmd_u32("var", unsigned.unwrap_or(1)),
            "cmd.common" => self.set_cmd_u32("common", unsigned.unwrap_or(1)),
            "cmd.wait" => self.set_cmd_u32("wait", unsigned.unwrap_or(0)),
            "cmd.map" => self.set_cmd_u32("map", unsigned.unwrap_or(1)),
            "cmd.x" => self.set_cmd_u32("x", unsigned.unwrap_or(0)),
            "cmd.y" => self.set_cmd_u32("y", unsigned.unwrap_or(0)),
            "cmd.amount" => self.set_cmd_i32("amount", signed),
            "cmd.item" => self.set_cmd_u32("item", unsigned.unwrap_or(1)),
            "cmd.actor" => self.set_cmd_u32("actor", unsigned.unwrap_or(1)),
            "cmd.event" => self.set_cmd_u32("event", unsigned.unwrap_or(1)),
            "cmd.picture" => self.set_cmd_u32("picture", unsigned.unwrap_or(1)),
            "cmd.px" => self.set_cmd_i32("px", signed),
            "cmd.py" => self.set_cmd_i32("py", signed),
            "cmd.frames" => self.set_cmd_u32("frames", unsigned.unwrap_or(1)),
            "cmd.troop" => self.set_cmd_u32("troop", unsigned.unwrap_or(1)),
            "cond.switch" => self.set_cond_num("switch", signed, unsigned.unwrap_or(1)),
            "cond.var" => self.set_cond_num("var", signed, unsigned.unwrap_or(1)),
            "cond.item" => self.set_cond_num("item", signed, unsigned.unwrap_or(1)),
            "cond.gold" => self.set_cond_num("gold", signed, unsigned.unwrap_or(0)),
            "cond.actor" => self.set_cond_num("actor", signed, unsigned.unwrap_or(1)),
            "cond.hp" => self.set_cond_num("hp", signed, unsigned.unwrap_or(1)),
            "cond.amount" => self.set_cond_num("amount", signed, unsigned.unwrap_or(0)),
            "cond.value" | "cond.id" | "cond.min" | "cond.max" => self.set_operand_num(
                true,
                key.trim_start_matches("cond."),
                signed,
                unsigned.unwrap_or(0),
            ),
            "var.value" | "var.id" | "var.min" | "var.max" => self.set_operand_num(
                false,
                key.trim_start_matches("var."),
                signed,
                unsigned.unwrap_or(0),
            ),
            _ => false,
        }
    }

    pub(crate) fn apply_pick(&mut self, key: &str, value: String) {
        if self.write_pick(key, &value) {
            self.changed();
        }
    }

    fn write_pick(&mut self, key: &str, value: &str) -> bool {
        if let Some(rest) = key.strip_prefix("effect.kind.") {
            return self.set_effect_kind(rest, value);
        }
        if let Some(rest) = key.strip_prefix("action.cond.") {
            return self.set_action_cond(rest, value);
        }
        match key {
            "map.tileset" => {
                let Some(id) = leading_id(value) else {
                    return false;
                };
                self.map_mut().map(|map| map.tileset_id = id).is_some()
            }
            "actor.class" => self
                .actor_mut()
                .map(|actor| actor.class_id = leading_id(value).unwrap_or(0))
                .is_some(),
            "actor.weapon" => self
                .actor_mut()
                .map(|actor| actor.weapon_id = leading_id(value).unwrap_or(0))
                .is_some(),
            "actor.shield" => self
                .actor_mut()
                .map(|actor| actor.shield_id = leading_id(value).unwrap_or(0))
                .is_some(),
            "actor.head" => self
                .actor_mut()
                .map(|actor| actor.head_id = leading_id(value).unwrap_or(0))
                .is_some(),
            "actor.body" => self
                .actor_mut()
                .map(|actor| actor.body_id = leading_id(value).unwrap_or(0))
                .is_some(),
            "actor.accessory" => self
                .actor_mut()
                .map(|actor| actor.accessory_id = leading_id(value).unwrap_or(0))
                .is_some(),
            "skill.occasion" => self
                .skill_mut()
                .and_then(|skill| by_label(&Occasion::all(), value).map(|v| skill.occasion = v))
                .is_some(),
            "skill.scope" => self
                .skill_mut()
                .and_then(|skill| by_label(&Scope::all(), value).map(|v| skill.scope = v))
                .is_some(),
            "skill.kind" => self
                .skill_mut()
                .and_then(|skill| {
                    by_label(&DamageKind::all(), value).map(|v| skill.damage.kind = v)
                })
                .is_some(),
            "skill.element" => self
                .skill_mut()
                .and_then(|skill| {
                    by_label(&fablespace_data::Element::all(), value)
                        .map(|v| skill.damage.element = v)
                })
                .is_some(),
            "item.occasion" => self
                .item_mut()
                .and_then(|item| by_label(&Occasion::all(), value).map(|v| item.occasion = v))
                .is_some(),
            "item.scope" => self
                .item_mut()
                .and_then(|item| by_label(&Scope::all(), value).map(|v| item.scope = v))
                .is_some(),
            "item.kind" => self
                .item_mut()
                .and_then(|item| by_label(&DamageKind::all(), value).map(|v| item.damage.kind = v))
                .is_some(),
            "item.element" => self
                .item_mut()
                .and_then(|item| {
                    by_label(&fablespace_data::Element::all(), value)
                        .map(|v| item.damage.element = v)
                })
                .is_some(),
            "armor.slot" => self
                .armor_mut()
                .and_then(|armor| by_label(&ArmorSlot::all(), value).map(|v| armor.slot = v))
                .is_some(),
            "state.restriction" => self
                .state_mut()
                .and_then(|state| {
                    by_label(&[Restriction::None, Restriction::CannotAct], value)
                        .map(|v| state.restriction = v)
                })
                .is_some(),
            "state.removal" => self.set_removal(value),
            "common.trigger" => self
                .common_mut()
                .and_then(|event| by_label(&CommonTrigger::all(), value).map(|v| event.trigger = v))
                .is_some(),
            "system.dir" => by_label(&Direction::all(), value)
                .map(|dir| self.game.system.start_dir = dir)
                .is_some(),
            "page.trigger" => self
                .page_mut()
                .and_then(|page| by_label(&Trigger::all(), value).map(|v| page.trigger = v))
                .is_some(),
            "page.priority" => self
                .page_mut()
                .and_then(|page| by_label(&Priority::all(), value).map(|v| page.priority = v))
                .is_some(),
            "page.move" => self
                .page_mut()
                .and_then(|page| by_label(&MoveType::all(), value).map(|v| page.move_type = v))
                .is_some(),
            "page.direction" => self
                .page_mut()
                .and_then(|page| {
                    by_label(&Direction::all(), value).map(|v| page.image.direction = v)
                })
                .is_some(),
            "page.self" => self.set_page_self(value),
            "troop.span" => self
                .troop_page_mut()
                .and_then(|page| {
                    by_label(&[Span::Battle, Span::Turn], value).map(|v| page.span = v)
                })
                .is_some(),
            "cmd.switch_value" => self.set_switch_value(value),
            "cmd.var_op" => self.set_var_op(value),
            "cmd.self" => self.set_self_key(value),
            "cmd.dir" => self.set_transfer_dir(value),
            "cmd.gold_op" | "cmd.item_op" | "cmd.hp_op" | "cmd.mp_op" => {
                self.set_amount_op(key, value)
            }
            "cmd.hp_target" | "cmd.mp_target" | "cmd.recover" => self.set_actor_target(key, value),
            "cmd.move" => self.set_move_target(value),
            "cmd.cancel" => self.set_cancel(value),
            "cond.kind" => self.set_cond_kind(value),
            "cond.cmp" => self.set_cond_cmp(value),
            "cond.self" => self.set_cond_self(value),
            "cond.operand" => self.set_operand_kind(true, value),
            "var.operand" => self.set_operand_kind(false, value),
            _ => false,
        }
    }

    pub(crate) fn apply_toggle(&mut self, key: &str, on: bool) {
        if let Some(rest) = key.strip_prefix("route.") {
            if self.set_route_toggle(rest, on) {
                self.changed();
            }
            return;
        }
        let wrote = match key {
            "item.consumable" => self.item_mut().map(|item| item.consumable = on).is_some(),
            "page.walk" => self.page_mut().map(|page| page.walk_anime = on).is_some(),
            "page.step" => self.page_mut().map(|page| page.step_anime = on).is_some(),
            "page.fix" => self
                .page_mut()
                .map(|page| page.direction_fix = on)
                .is_some(),
            "page.through" => self.page_mut().map(|page| page.through = on).is_some(),
            "cmd.else" => {
                self.update_cmd(|cmd| {
                    command::set_else(cmd, on);
                    matches!(cmd, Command::Conditional { .. })
                });
                return;
            }
            "cmd.fade" => {
                self.update_cmd(|cmd| {
                    if let Command::TransferPlayer { fade, .. } = cmd {
                        *fade = on;
                        true
                    } else {
                        false
                    }
                });
                return;
            }
            "cmd.escape" => {
                self.update_cmd(|cmd| {
                    if let Command::Battle { can_escape, .. } = cmd {
                        *can_escape = on;
                        true
                    } else {
                        false
                    }
                });
                return;
            }
            "cmd.self_on" => {
                self.update_cmd(|cmd| {
                    if let Command::ControlSelfSwitch { on: slot, .. } = cmd {
                        *slot = on;
                        true
                    } else {
                        false
                    }
                });
                return;
            }
            "cond.on" => self.set_cond_on(on),
            _ => false,
        };
        if wrote {
            self.changed();
        }
    }

    pub(crate) fn apply_route(&mut self, owner: &str, op: &str, index: usize) {
        match op {
            "add" => {
                let Some(step) = MoveStep::catalog().get(index).cloned() else {
                    return;
                };
                self.with_route(owner, |route| route.steps.push(step));
            }
            "del" => self.with_route(owner, |route| {
                if index < route.steps.len() {
                    route.steps.remove(index);
                }
            }),
            _ => {}
        }
    }

    pub(crate) fn apply_stack(&mut self, kind: &str, op: &str, index: usize) {
        let list = match kind {
            "item" => &mut self.game.system.start_items,
            "weapon" => &mut self.game.system.start_weapons,
            "armor" => &mut self.game.system.start_armors,
            _ => return,
        };
        match op {
            "add" => list.push(Stack { id: 1, count: 1 }),
            "del" => {
                if index < list.len() {
                    list.remove(index);
                }
            }
            _ => return,
        }
        self.num_key.clear();
        self.changed();
    }

    pub(crate) fn list_add(&mut self, key: &str) {
        let wrote = match key {
            "learning" => self
                .class_mut()
                .map(|class| {
                    class.learnings.push(Learning {
                        level: 1,
                        skill_id: 1,
                    })
                })
                .is_some(),
            "skill.effect" => self
                .skill_mut()
                .map(|skill| {
                    skill.effects.push(Effect::RecoverHp {
                        percent: 0,
                        flat: 10,
                    })
                })
                .is_some(),
            "item.effect" => self
                .item_mut()
                .map(|item| {
                    item.effects.push(Effect::RecoverHp {
                        percent: 0,
                        flat: 10,
                    })
                })
                .is_some(),
            "drop" => self
                .enemy_mut()
                .map(|enemy| {
                    enemy.drops.push(Drop {
                        item_id: 1,
                        denominator: 1,
                    })
                })
                .is_some(),
            "action" => self
                .enemy_mut()
                .map(|enemy| {
                    enemy.actions.push(EnemyAction {
                        skill_id: 0,
                        rating: 5,
                        condition: ActionCondition::Always,
                    })
                })
                .is_some(),
            "member" => self
                .troop_mut()
                .map(|troop| {
                    troop.members.push(TroopMember {
                        enemy_id: 1,
                        x: 220,
                        y: 120,
                    })
                })
                .is_some(),
            "autotile" => self
                .tileset_mut()
                .map(|tileset| {
                    tileset.autotiles.push(Autotile {
                        tile_id: 8,
                        origin_col: 0,
                        origin_row: 1,
                    })
                })
                .is_some(),
            _ => false,
        };
        if wrote {
            self.num_key.clear();
            self.changed();
        }
    }

    pub(crate) fn list_del(&mut self, key: &str, index: usize) {
        let wrote = match key {
            "learning" => remove_at(self.class_mut().map(|class| &mut class.learnings), index),
            "skill.effect" => remove_at(self.skill_mut().map(|skill| &mut skill.effects), index),
            "item.effect" => remove_at(self.item_mut().map(|item| &mut item.effects), index),
            "drop" => remove_at(self.enemy_mut().map(|enemy| &mut enemy.drops), index),
            "action" => remove_at(self.enemy_mut().map(|enemy| &mut enemy.actions), index),
            "member" => remove_at(self.troop_mut().map(|troop| &mut troop.members), index),
            "autotile" => remove_at(
                self.tileset_mut().map(|tileset| &mut tileset.autotiles),
                index,
            ),
            _ => false,
        };
        if wrote {
            self.num_key.clear();
            self.changed();
        }
    }

    pub(crate) fn cycle_passage(&mut self, id: u32) {
        let Some(tileset) = self.tileset_mut() else {
            return;
        };
        let need = id as usize + 1;
        while tileset.tiles.len() < need {
            tileset.tiles.push(TileDef {
                passage: Passage::Pass,
            });
        }
        let passage = &mut tileset.tiles[id as usize].passage;
        *passage = passage.cycle_upper();
        self.changed();
    }

    pub(crate) fn add_page(&mut self) {
        match self.screen {
            Screen::Map => {
                let next = self.event_mut().map(|event| {
                    event.pages.push(Page::default());
                    event.pages.len() - 1
                });
                if let Some(next) = next {
                    self.page = next;
                    self.cmd_target = None;
                    self.changed();
                }
            }
            Screen::Troops => {
                let id = self.sel;
                let next = self
                    .game
                    .troops
                    .get_mut(id as usize)
                    .filter(|_| id != 0)
                    .map(|troop| {
                        troop.pages.push(TroopPage::default());
                        troop.pages.len() - 1
                    });
                if let Some(next) = next {
                    self.page = next;
                    self.cmd_target = None;
                    self.changed();
                }
            }
            _ => {}
        }
    }

    pub(crate) fn delete_page(&mut self) {
        match self.screen {
            Screen::Map => {
                let page = self.page;
                let len = self.event_mut().and_then(|event| {
                    if event.pages.len() <= 1 {
                        return None;
                    }
                    event.pages.remove(page.min(event.pages.len() - 1));
                    Some(event.pages.len())
                });
                if let Some(len) = len {
                    self.page = self.page.min(len.saturating_sub(1));
                    self.cmd_target = None;
                    self.changed();
                }
            }
            Screen::Troops => {
                let id = self.sel;
                let page = self.page;
                let len = self
                    .game
                    .troops
                    .get_mut(id as usize)
                    .filter(|_| id != 0)
                    .and_then(|troop| {
                        if page < troop.pages.len() {
                            troop.pages.remove(page);
                        }
                        Some(troop.pages.len())
                    });
                if let Some(len) = len {
                    self.page = if len == 0 { 0 } else { self.page.min(len - 1) };
                    self.cmd_target = None;
                    self.changed();
                }
            }
            _ => {}
        }
    }

    pub(crate) fn command_add(&mut self) {
        let kind = Command::kinds()
            .get(self.kind)
            .map(|item| item.0)
            .unwrap_or("show_text");
        let target = self.cmd_target.clone();
        let placed = {
            let Some(list) = self.commands_mut() else {
                self.status = "Open a page that has a command list.".into();
                return;
            };
            match &target {
                Some(Target::Command(path)) => {
                    command::insert_after(list, path, Command::blank(kind))
                }
                Some(Target::Branch { parent, slot }) => {
                    command::insert_in_branch(list, parent, *slot, Command::blank(kind))
                }
                None => {
                    list.push(Command::blank(kind));
                    true
                }
            }
        };
        if placed {
            self.follow_insert(&target);
            self.changed();
        } else {
            self.status = "Choose a command or a branch, then add.".into();
        }
    }

    pub(crate) fn command_remove(&mut self) {
        let Some(Target::Command(path)) = self.cmd_target.clone() else {
            return;
        };
        let removed = self
            .commands_mut()
            .is_some_and(|list| command::remove_command(list, &path));
        if removed {
            self.cmd_target = None;
            self.changed();
        }
    }

    pub(crate) fn command_move(&mut self, delta: isize) {
        let Some(Target::Command(mut path)) = self.cmd_target.clone() else {
            return;
        };
        let moved = self
            .commands_mut()
            .is_some_and(|list| command::move_command(list, &path, delta));
        if moved {
            if let Some(Step::Cmd(index)) = path.last_mut() {
                *index = (*index as isize + delta) as usize;
            }
            self.cmd_target = Some(Target::Command(path));
            self.changed();
        }
    }

    pub(crate) fn choice_add(&mut self) {
        self.update_cmd(|cmd| {
            if let Command::ShowChoices { choices, .. } = cmd {
                choices.push(format!("Choice {}", choices.len() + 1));
                command::sync_choice_branches(cmd);
                true
            } else {
                false
            }
        });
    }

    pub(crate) fn choice_remove(&mut self) {
        self.update_cmd(|cmd| {
            let Command::ShowChoices {
                choices, cancel, ..
            } = cmd
            else {
                return false;
            };
            if choices.len() <= 1 {
                return false;
            }
            choices.pop();
            if let CancelMode::Choice(index) = *cancel {
                if index >= choices.len() {
                    *cancel = CancelMode::Disallow;
                }
            }
            command::sync_choice_branches(cmd);
            true
        });
    }

    fn follow_insert(&mut self, target: &Option<Target>) {
        let Some(list) = self.command_slice() else {
            return;
        };
        match target {
            Some(Target::Command(path)) => {
                let mut path = path.clone();
                if let Some(Step::Cmd(index)) = path.last_mut() {
                    *index += 1;
                }
                self.cmd_target = Some(Target::Command(path));
            }
            Some(Target::Branch { parent, slot }) => {
                let step = slot_step(*slot);
                let mut best = None;
                for row in command::rows(list) {
                    if let Target::Command(path) = &row.target {
                        if path.len() == parent.len() + 2
                            && path[..parent.len()] == parent[..]
                            && path[parent.len()] == step
                        {
                            best = Some(path.clone());
                        }
                    }
                }
                if let Some(path) = best {
                    self.cmd_target = Some(Target::Command(path));
                }
            }
            None => {
                if !list.is_empty() {
                    self.cmd_target = Some(Target::Command(vec![Step::Cmd(list.len() - 1)]));
                }
            }
        }
    }

    pub(crate) fn database_view(&self) -> Element<'_, Message> {
        let form = iced::widget::scrollable(chrome::card(self.screen.label(), self.detail()))
            .height(Length::Fill)
            .width(Length::Fill);
        if !self.screen.listed() {
            return form.into();
        }
        let mut side = Column::new().spacing(6);
        side = side.push(
            row![
                chrome::action("Add", Some(Message::Add)),
                chrome::action("Delete last", Some(Message::Delete)),
                chrome::action("Clear", Some(Message::Clear)),
            ]
            .spacing(6),
        );
        for (id, name) in self.names() {
            side = side.push(chrome::nav_item(
                format!("{id}  {name}"),
                id == self.sel,
                Message::Select(id),
            ));
        }
        row![
            chrome::card(
                "Entries",
                iced::widget::scrollable(side).height(Length::Fill).into(),
            ),
            form
        ]
        .spacing(16)
        .into()
    }

    pub(crate) fn event_section(&self) -> Element<'_, Message> {
        let Some(map) = self.map() else {
            return text("").into();
        };
        let Some(event_id) = self.event else {
            return text(
                "Event tool: click a tile to place an event, or click an existing one to edit it.",
            )
            .into();
        };
        let Some(event) = map.events.iter().find(|event| event.id == event_id) else {
            return text("That event is no longer on this map.").into();
        };
        let identity = column![
            field_text("event.name", "Name", &event.name),
            row![
                field_num(self, "event.x", "X", event.x),
                field_num(self, "event.y", "Y", event.y),
                chrome::action("Delete event", Some(Message::DeleteEvent)),
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center),
        ]
        .spacing(8);
        let mut pages = row![].spacing(6);
        for index in 0..event.pages.len() {
            pages = pages.push(chrome::nav_item(
                format!("Page {}", index + 1),
                index == self.page,
                Message::ChoosePage(index),
            ));
        }
        let pages = column![
            pages,
            row![
                chrome::action("Add page", Some(Message::AddPage)),
                chrome::action("Delete page", Some(Message::DeletePage)),
            ]
            .spacing(8),
        ]
        .spacing(8);
        let mut stack = Column::new().spacing(12);
        stack = stack.push(chrome::card(
            &format!("Event {} · {}", event.id, event.name),
            identity.into(),
        ));
        stack = stack.push(chrome::card("Pages", pages.into()));
        if let Some(page) = event.pages.get(self.page) {
            let conditions = column![
                text("Use 0 when a condition is unused.")
                    .size(13)
                    .color(chrome::MUTED),
                row![
                    field_num(self, "page.sw1", "Switch 1", page.conditions.switch_1),
                    field_num(self, "page.sw2", "Switch 2", page.conditions.switch_2),
                ]
                .spacing(8),
                row![
                    field_num(self, "page.var", "Variable", page.conditions.variable_id),
                    field_num(
                        self,
                        "page.varmin",
                        "At least",
                        page.conditions.variable_min
                    ),
                ]
                .spacing(8),
                field_pick(
                    "page.self",
                    "Self switch",
                    self_options(),
                    self_current(page.conditions.self_switch),
                ),
                row![
                    field_num(self, "page.item", "Item", page.conditions.item_id),
                    field_num(self, "page.actor", "Actor", page.conditions.actor_id),
                ]
                .spacing(8),
            ]
            .spacing(8);
            let appearance = column![
                field_text("page.character", "Character", &page.image.character),
                row![
                    field_num(self, "page.index", "Index", page.image.index),
                    field_pick(
                        "page.direction",
                        "Direction",
                        labels(&Direction::all()),
                        page.image.direction.to_string()
                    ),
                    field_num(self, "page.pattern", "Pattern", page.image.pattern),
                ]
                .spacing(8),
                row![
                    field_pick(
                        "page.trigger",
                        "Trigger",
                        labels(&Trigger::all()),
                        page.trigger.to_string()
                    ),
                    field_pick(
                        "page.priority",
                        "Priority",
                        labels(&Priority::all()),
                        page.priority.to_string()
                    ),
                ]
                .spacing(8),
            ]
            .spacing(8);
            let mut movement = column![
                row![
                    field_pick(
                        "page.move",
                        "Move",
                        labels(&MoveType::all()),
                        page.move_type.to_string()
                    ),
                    field_num(self, "page.speed", "Speed", page.move_speed),
                    field_num(self, "page.frequency", "Frequency", page.move_frequency),
                ]
                .spacing(8),
                row![
                    check("page.walk", "Walk animation", page.walk_anime),
                    check("page.step", "Step animation", page.step_anime),
                    check("page.fix", "Direction fix", page.direction_fix),
                    check("page.through", "Through", page.through),
                ]
                .spacing(8),
            ]
            .spacing(8);
            if page.move_type == MoveType::Custom {
                movement = movement.push(text("Custom route").size(15).color(chrome::GOLD));
                movement = movement.push(self.route_editor("page", &page.route));
            }
            stack = stack.push(chrome::card("Conditions", conditions.into()));
            stack = stack.push(chrome::card("Appearance", appearance.into()));
            stack = stack.push(chrome::card("Movement", movement.into()));
        }
        stack = stack.push(chrome::card("Commands", self.command_editor()));
        stack.into()
    }

    fn detail(&self) -> Element<'_, Message> {
        match self.screen {
            Screen::Actors => self.actor_form(),
            Screen::Classes => self.class_form(),
            Screen::Skills => self.skill_form(),
            Screen::Items => self.item_form(),
            Screen::Weapons => self.weapon_form(),
            Screen::Armors => self.armor_form(),
            Screen::Enemies => self.enemy_form(),
            Screen::Troops => self.troop_form(),
            Screen::States => self.state_form(),
            Screen::Tilesets => self.tileset_form(),
            Screen::Commons => self.common_form(),
            Screen::Switches => self.name_list("switch", &self.game.switches, "Switch"),
            Screen::Variables => self.name_list("variable", &self.game.variables, "Variable"),
            Screen::System => self.system_form(),
            Screen::Map => text("").into(),
        }
    }

    fn names(&self) -> Vec<(u32, String)> {
        match self.screen {
            Screen::Actors => named(&self.game.actors, |a| a.id, |a| a.name.clone()),
            Screen::Classes => named(&self.game.classes, |a| a.id, |a| a.name.clone()),
            Screen::Skills => named(&self.game.skills, |a| a.id, |a| a.name.clone()),
            Screen::Items => named(&self.game.items, |a| a.id, |a| a.name.clone()),
            Screen::Weapons => named(&self.game.weapons, |a| a.id, |a| a.name.clone()),
            Screen::Armors => named(&self.game.armors, |a| a.id, |a| a.name.clone()),
            Screen::Enemies => named(&self.game.enemies, |a| a.id, |a| a.name.clone()),
            Screen::Troops => named(&self.game.troops, |a| a.id, |a| a.name.clone()),
            Screen::States => named(&self.game.states, |a| a.id, |a| a.name.clone()),
            Screen::Tilesets => named(&self.game.tilesets, |a| a.id, |a| a.name.clone()),
            Screen::Commons => named(&self.game.common_events, |a| a.id, |a| a.name.clone()),
            _ => Vec::new(),
        }
    }

    fn actor_form(&self) -> Element<'_, Message> {
        let Some(actor) = self
            .game
            .actors
            .get(self.sel as usize)
            .filter(|actor| actor.id != 0)
        else {
            return missing();
        };
        let mut body = Column::new().spacing(8);
        body = body.push(heading(&format!("Actor {}", actor.id)));
        body = body.push(field_text("actor.name", "Name", &actor.name));
        body = body.push(field_text("actor.nickname", "Nickname", &actor.nickname));
        body = body.push(field_pick(
            "actor.class",
            "Class",
            id_options(self.game.classes.iter().map(|c| (c.id, c.name.clone()))),
            id_current(actor.class_id, &self.lookup_name(actor.class_id, "class")),
        ));
        body = body.push(
            row![
                field_num(self, "actor.level", "Initial level", actor.initial_level),
                field_num(self, "actor.max_level", "Max level", actor.max_level)
            ]
            .spacing(8),
        );
        body = body.push(field_text("actor.profile", "Profile", &actor.profile));
        body = body.push(field_text("actor.face", "Face", &actor.face));
        body = body.push(row![btn("Import face", Message::Import("faces".into()))].spacing(8));
        body = body.push(field_text("actor.character", "Character", &actor.character));
        body = body.push(field_num(
            self,
            "actor.char_index",
            "Character index",
            actor.character_index,
        ));
        body = body.push(btn(
            "Import character",
            Message::Import("characters".into()),
        ));
        body = body.push(field_pick(
            "actor.weapon",
            "Weapon",
            id_options(self.game.weapons.iter().map(|w| (w.id, w.name.clone()))),
            id_current(
                actor.weapon_id,
                &self.lookup_name(actor.weapon_id, "weapon"),
            ),
        ));
        body = body.push(field_pick(
            "actor.shield",
            "Shield",
            id_options(
                self.game
                    .armors
                    .iter()
                    .filter(|a| a.slot == ArmorSlot::Shield)
                    .map(|a| (a.id, a.name.clone())),
            ),
            id_current(actor.shield_id, &self.lookup_name(actor.shield_id, "armor")),
        ));
        body = body.push(field_pick(
            "actor.head",
            "Head",
            id_options(
                self.game
                    .armors
                    .iter()
                    .filter(|a| a.slot == ArmorSlot::Head)
                    .map(|a| (a.id, a.name.clone())),
            ),
            id_current(actor.head_id, &self.lookup_name(actor.head_id, "armor")),
        ));
        body = body.push(field_pick(
            "actor.body",
            "Body",
            id_options(
                self.game
                    .armors
                    .iter()
                    .filter(|a| a.slot == ArmorSlot::Body)
                    .map(|a| (a.id, a.name.clone())),
            ),
            id_current(actor.body_id, &self.lookup_name(actor.body_id, "armor")),
        ));
        body = body.push(field_pick(
            "actor.accessory",
            "Accessory",
            id_options(
                self.game
                    .armors
                    .iter()
                    .filter(|a| a.slot == ArmorSlot::Accessory)
                    .map(|a| (a.id, a.name.clone())),
            ),
            id_current(
                actor.accessory_id,
                &self.lookup_name(actor.accessory_id, "armor"),
            ),
        ));
        body = body.push(field_text("actor.note", "Note", &actor.note));
        body.into()
    }

    fn class_form(&self) -> Element<'_, Message> {
        let Some(class) = self
            .game
            .classes
            .get(self.sel as usize)
            .filter(|class| class.id != 0)
        else {
            return missing();
        };
        let mut body = Column::new().spacing(8);
        body = body.push(heading(&format!("Class {}", class.id)));
        body = body.push(field_text("class.name", "Name", &class.name));
        body = body.push(
            row![
                field_num(self, "class.basis", "Exp basis", class.exp.basis),
                field_num(self, "class.extra", "Extra", class.exp.extra),
                field_num(self, "class.accel", "Accel", class.exp.accel),
            ]
            .spacing(8),
        );
        body = body.push(text("Parameters are base plus growth times (level - 1).").size(14));
        for index in 0..PARAM_COUNT {
            let growth = class.growth(index);
            body = body.push(
                row![
                    text(PARAM_LABELS[index]).width(Length::Fixed(120.0)),
                    field_num(self, &format!("class.base.{index}"), "Base", growth.base),
                    field_num(
                        self,
                        &format!("class.plus.{index}"),
                        "Per level",
                        growth.plus
                    ),
                ]
                .spacing(8),
            );
        }
        body = body.push(text("Learnings").size(18));
        for (index, learning) in class.learnings.iter().enumerate() {
            body = body.push(
                row![
                    field_num(
                        self,
                        &format!("learning.level.{index}"),
                        "Level",
                        learning.level
                    ),
                    field_num(
                        self,
                        &format!("learning.skill.{index}"),
                        "Skill",
                        learning.skill_id
                    ),
                    btn(
                        "Remove",
                        Message::ListDel {
                            key: "learning".into(),
                            index
                        }
                    ),
                ]
                .spacing(8),
            );
        }
        body = body.push(btn(
            "Add learning",
            Message::ListAdd {
                key: "learning".into(),
            },
        ));
        body = body.push(field_text("class.note", "Note", &class.note));
        body.into()
    }

    fn skill_form(&self) -> Element<'_, Message> {
        let Some(skill) = self
            .game
            .skills
            .get(self.sel as usize)
            .filter(|skill| skill.id != 0)
        else {
            return missing();
        };
        let mut body = self.usage_form(
            "Skill",
            skill.id,
            &skill.name,
            &skill.description,
            &skill.note,
            "skill",
            skill.occasion,
            skill.scope,
            &skill.damage,
            skill.hit,
            skill.mp_cost,
            "MP",
        );
        body = push_effects(body, self, "skill", &skill.effects);
        body.into()
    }

    fn item_form(&self) -> Element<'_, Message> {
        let Some(item) = self
            .game
            .items
            .get(self.sel as usize)
            .filter(|item| item.id != 0)
        else {
            return missing();
        };
        let mut body = self.usage_form(
            "Item",
            item.id,
            &item.name,
            &item.description,
            &item.note,
            "item",
            item.occasion,
            item.scope,
            &item.damage,
            item.hit,
            item.price,
            "Price",
        );
        body = body.push(check("item.consumable", "Consumed on use", item.consumable));
        body = push_effects(body, self, "item", &item.effects);
        body.into()
    }

    fn weapon_form(&self) -> Element<'_, Message> {
        let Some(weapon) = self
            .game
            .weapons
            .get(self.sel as usize)
            .filter(|weapon| weapon.id != 0)
        else {
            return missing();
        };
        let mut body = gear_header(
            "Weapon",
            weapon.id,
            &weapon.name,
            &weapon.description,
            &weapon.note,
            "weapon",
        );
        body = body.push(field_num(self, "weapon.price", "Price", weapon.price));
        body = push_params(body, self, "weapon.param", &weapon.params);
        body.into()
    }

    fn armor_form(&self) -> Element<'_, Message> {
        let Some(armor) = self
            .game
            .armors
            .get(self.sel as usize)
            .filter(|armor| armor.id != 0)
        else {
            return missing();
        };
        let mut body = gear_header(
            "Armor",
            armor.id,
            &armor.name,
            &armor.description,
            &armor.note,
            "armor",
        );
        body = body.push(field_num(self, "armor.price", "Price", armor.price));
        body = body.push(field_pick(
            "armor.slot",
            "Slot",
            labels(&ArmorSlot::all()),
            armor.slot.to_string(),
        ));
        body = push_params(body, self, "armor.param", &armor.params);
        body.into()
    }

    fn enemy_form(&self) -> Element<'_, Message> {
        let Some(enemy) = self
            .game
            .enemies
            .get(self.sel as usize)
            .filter(|enemy| enemy.id != 0)
        else {
            return missing();
        };
        let mut body = Column::new().spacing(8);
        body = body.push(heading(&format!("Enemy {}", enemy.id)));
        body = body.push(field_text("enemy.name", "Name", &enemy.name));
        body = body.push(field_text("enemy.battler", "Battler", &enemy.battler));
        body = body.push(btn("Import battler", Message::Import("enemies".into())));
        body = push_params(body, self, "enemy.param", &enemy.params);
        body = body.push(
            row![
                field_num(self, "enemy.exp", "Exp", enemy.exp),
                field_num(self, "enemy.gold", "Gold", enemy.gold)
            ]
            .spacing(8),
        );
        body = body.push(text("Drops. Denominator 1 is certain. 2 is one in two.").size(14));
        for (index, drop) in enemy.drops.iter().enumerate() {
            body = body.push(
                row![
                    field_num(self, &format!("drop.item.{index}"), "Item", drop.item_id),
                    field_num(self, &format!("drop.den.{index}"), "1 in", drop.denominator),
                    btn(
                        "Remove",
                        Message::ListDel {
                            key: "drop".into(),
                            index
                        }
                    ),
                ]
                .spacing(8),
            );
        }
        body = body.push(btn("Add drop", Message::ListAdd { key: "drop".into() }));
        body =
            body.push(text("Actions. Skill 0 is a basic attack. Rating is the weight.").size(14));
        for (index, action) in enemy.actions.iter().enumerate() {
            body = body.push(
                row![
                    field_num(
                        self,
                        &format!("action.skill.{index}"),
                        "Skill",
                        action.skill_id
                    ),
                    field_num(
                        self,
                        &format!("action.rating.{index}"),
                        "Rating",
                        action.rating
                    ),
                    field_pick(
                        &format!("action.cond.{index}"),
                        "When",
                        action_labels(),
                        action_current(&action.condition)
                    ),
                ]
                .spacing(8),
            );
            body = body.push(action_fields(self, index, &action.condition));
            body = body.push(btn(
                "Remove action",
                Message::ListDel {
                    key: "action".into(),
                    index,
                },
            ));
        }
        body = body.push(btn(
            "Add action",
            Message::ListAdd {
                key: "action".into(),
            },
        ));
        body = body.push(field_text("enemy.note", "Note", &enemy.note));
        body.into()
    }

    fn troop_form(&self) -> Element<'_, Message> {
        let Some(troop) = self
            .game
            .troops
            .get(self.sel as usize)
            .filter(|troop| troop.id != 0)
        else {
            return missing();
        };
        let mut body = Column::new().spacing(8);
        body = body.push(heading(&format!("Troop {}", troop.id)));
        body = body.push(field_text("troop.name", "Name", &troop.name));
        for (index, member) in troop.members.iter().enumerate() {
            body = body.push(
                row![
                    field_num(
                        self,
                        &format!("member.enemy.{index}"),
                        "Enemy",
                        member.enemy_id
                    ),
                    field_num(self, &format!("member.x.{index}"), "X", member.x),
                    field_num(self, &format!("member.y.{index}"), "Y", member.y),
                    btn(
                        "Remove",
                        Message::ListDel {
                            key: "member".into(),
                            index
                        }
                    ),
                ]
                .spacing(8),
            );
        }
        body = body.push(btn(
            "Add member",
            Message::ListAdd {
                key: "member".into(),
            },
        ));
        let mut pages = row![].spacing(4);
        for index in 0..troop.pages.len() {
            let label = if index == self.page {
                format!("▸ Page {}", index + 1)
            } else {
                format!("Page {}", index + 1)
            };
            pages = pages.push(button(text(label)).on_press(Message::ChoosePage(index)));
        }
        body = body.push(pages);
        body = body.push(
            row![
                btn("Add page", Message::AddPage),
                btn("Delete page", Message::DeletePage)
            ]
            .spacing(8),
        );
        if let Some(page) = troop.pages.get(self.page) {
            let turn = page.condition.turn_start.unwrap_or(0);
            body = body.push(text("Turn 0 with every 0 means the condition is not a turn check. A member index with HP 0 means no HP check.").size(14));
            body = body.push(
                row![
                    field_num(self, "troop.turn", "Turn", turn),
                    field_num(self, "troop.every", "Every", page.condition.turn_every),
                    field_pick(
                        "troop.span",
                        "Span",
                        labels(&[Span::Battle, Span::Turn]),
                        page.span.to_string()
                    ),
                ]
                .spacing(8),
            );
            let (member, hp) = page.condition.enemy_hp.unwrap_or((0, 0));
            body = body.push(
                row![
                    field_num(self, "troop.member", "Member", member as u32),
                    field_num(self, "troop.hp", "HP %", hp),
                    field_num(self, "troop.switch", "Switch", page.condition.switch_id),
                ]
                .spacing(8),
            );
            body = body.push(self.command_editor());
        } else {
            body = body.push(text("This troop has no event pages."));
        }
        body.into()
    }

    fn state_form(&self) -> Element<'_, Message> {
        let Some(state) = self
            .game
            .states
            .get(self.sel as usize)
            .filter(|state| state.id != 0)
        else {
            return missing();
        };
        let mut body = Column::new().spacing(8);
        body = body.push(heading(&format!("State {}", state.id)));
        body = body.push(field_text("state.name", "Name", &state.name));
        body = body.push(field_pick(
            "state.restriction",
            "Restriction",
            labels(&[Restriction::None, Restriction::CannotAct]),
            state.restriction.to_string(),
        ));
        body = body.push(field_pick(
            "state.removal",
            "Removal",
            vec!["None".into(), "After turns".into(), "On damage".into()],
            removal_label(&state.removal),
        ));
        if let Removal::Turns { min, max } = state.removal {
            body = body.push(
                row![
                    field_num(self, "state.turn_min", "Min turns", min),
                    field_num(self, "state.turn_max", "Max turns", max)
                ]
                .spacing(8),
            );
        }
        body = body.push(field_num(
            self,
            "state.slip",
            "Slip % of max HP",
            state.slip_hp_percent,
        ));
        body = body.push(field_text("state.note", "Note", &state.note));
        body.into()
    }

    fn tileset_form(&self) -> Element<'_, Message> {
        let Some(tileset) = self
            .game
            .tilesets
            .get(self.sel as usize)
            .filter(|tileset| tileset.id != 0)
        else {
            return missing();
        };
        let mut body = Column::new().spacing(8);
        body = body.push(heading(&format!("Tileset {}", tileset.id)));
        body = body.push(field_text("tileset.name", "Name", &tileset.name));
        body = body.push(field_text("tileset.image", "Image", &tileset.image));
        body = body.push(btn("Import tileset", Message::Import("tilesets".into())));
        body = body.push(field_num(
            self,
            "tileset.columns",
            "Columns",
            tileset.columns,
        ));
        body = body.push(text("Pass walks. Block stops the player. Star draws above the player and never blocks. On the upper layer, Pass is under the player and Block sorts with them.").size(14));
        let count = tileset.tiles.len().max(1) as u32;
        for id in 0..count {
            let passage = tileset.passage(id);
            body = body.push(
                button(text(format!("Tile {id}: {}", passage.label())))
                    .on_press(Message::CyclePassage(id)),
            );
        }
        body = body
            .push(text("Autotiles. The painted id is the top-left of a 4 by 4 block.").size(14));
        for (index, auto) in tileset.autotiles.iter().enumerate() {
            body = body.push(
                row![
                    field_num(self, &format!("auto.tile.{index}"), "Tile", auto.tile_id),
                    field_num(
                        self,
                        &format!("auto.col.{index}"),
                        "Column",
                        auto.origin_col
                    ),
                    field_num(self, &format!("auto.row.{index}"), "Row", auto.origin_row),
                    btn(
                        "Remove",
                        Message::ListDel {
                            key: "autotile".into(),
                            index
                        }
                    ),
                ]
                .spacing(8),
            );
        }
        body = body.push(btn(
            "Add autotile",
            Message::ListAdd {
                key: "autotile".into(),
            },
        ));
        body = body.push(field_text("tileset.note", "Note", &tileset.note));
        body.into()
    }

    fn common_form(&self) -> Element<'_, Message> {
        let Some(event) = self
            .game
            .common_events
            .get(self.sel as usize)
            .filter(|event| event.id != 0)
        else {
            return missing();
        };
        let mut body = Column::new().spacing(8);
        body = body.push(heading(&format!("Common event {}", event.id)));
        body = body.push(field_text("common.name", "Name", &event.name));
        body = body.push(field_pick(
            "common.trigger",
            "Trigger",
            labels(&CommonTrigger::all()),
            event.trigger.to_string(),
        ));
        body = body.push(field_num(self, "common.switch", "Switch", event.switch_id));
        body = body.push(
            text("Autorun and Parallel start while the switch is on. None runs only when called.")
                .size(14),
        );
        body = body.push(self.command_editor());
        body.into()
    }

    fn system_form(&self) -> Element<'_, Message> {
        let system = &self.game.system;
        let mut body = Column::new().spacing(8);
        body = body.push(heading("System"));
        body = body.push(field_text("system.title", "Title", &system.game_title));
        body = body.push(field_text("system.currency", "Currency", &system.currency));
        body = body.push(field_text(
            "system.title_image",
            "Title image",
            &system.title_image,
        ));
        body = body.push(btn("Import title", Message::Import("titles".into())));
        body = body.push(
            row![
                field_num(self, "system.start_map", "Start map", system.start_map_id),
                field_num(self, "system.start_x", "X", system.start_x),
                field_num(self, "system.start_y", "Y", system.start_y),
                field_pick(
                    "system.dir",
                    "Direction",
                    labels(&Direction::all()),
                    system.start_dir.to_string()
                ),
            ]
            .spacing(8),
        );
        body = body.push(field_num(
            self,
            "system.start_gold",
            "Starting gold",
            system.start_gold,
        ));
        let party = system
            .party
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        body = body.push(field_text("system.party", "Party ids", &party));
        body = body.push(text("Starting items").size(18));
        body = push_stacks(body, self, "item", &system.start_items);
        body = body.push(text("Starting weapons").size(18));
        body = push_stacks(body, self, "weapon", &system.start_weapons);
        body = body.push(text("Starting armors").size(18));
        body = push_stacks(body, self, "armor", &system.start_armors);
        body = body.push(text("Terms").size(18));
        for (key, label, value) in term_fields(&system.terms) {
            body = body.push(field_text(key, label, value));
        }
        body.into()
    }

    fn name_list(&self, prefix: &str, names: &[String], title: &str) -> Element<'_, Message> {
        let mut body = Column::new().spacing(6);
        body = body.push(heading(title));
        body = body.push(text("Index 0 is unused. New names are added at the end, and only the last one can be removed.").size(14));
        body = body.push(
            row![
                btn("Add", Message::Add),
                btn("Delete Last", Message::Delete)
            ]
            .spacing(8),
        );
        for (index, name) in names.iter().enumerate().skip(1) {
            body = body.push(field_text(
                &format!("{prefix}.{index}"),
                &format!("{index}"),
                name,
            ));
        }
        body.into()
    }

    fn command_editor(&self) -> Element<'_, Message> {
        let mut body = Column::new().spacing(6);
        body = body.push(text("Commands").size(18));
        let Some(list) = self.command_slice() else {
            body = body.push(text("This page has no command list."));
            return body.into();
        };
        let rows = command::rows(list);
        if rows.is_empty() {
            body = body.push(text("No commands yet."));
        }
        for (index, row) in rows.iter().enumerate() {
            let indent = "    ".repeat(row.depth as usize);
            let mark = self.cmd_target.as_ref() == Some(&row.target);
            let prefix = if mark { "▸ " } else { "" };
            body = body.push(
                button(text(format!("{prefix}{indent}{}", row.label)).size(14))
                    .on_press(Message::CmdRow(index))
                    .width(Length::Fill),
            );
        }
        let kinds: Vec<String> = Command::kinds()
            .iter()
            .map(|(_, label)| (*label).to_string())
            .collect();
        let current = kinds.get(self.kind).cloned();
        body = body.push(pick_list(kinds, current, |value| {
            let index = Command::kinds()
                .iter()
                .position(|(_, label)| *label == value)
                .unwrap_or(0);
            Message::CmdKind(index)
        }));
        body = body.push(
            row![
                btn("Add", Message::CmdAdd),
                btn("Remove", Message::CmdRemove),
                btn("Up", Message::CmdUp),
                btn("Down", Message::CmdDown),
            ]
            .spacing(6),
        );
        if let Some(command) = self.selected_command() {
            body = body.push(self.command_form(&command));
        } else if matches!(self.cmd_target, Some(Target::Branch { .. })) {
            body = body.push(text(
                "Add places the next command inside the selected branch.",
            ));
        }
        body.into()
    }

    fn command_form(&self, cmd: &Command) -> Element<'_, Message> {
        let mut body = Column::new().spacing(6);
        body = body.push(text(command::summary(cmd)).size(16));
        match cmd {
            Command::ShowText {
                face,
                speaker,
                lines,
            } => {
                body = body.push(field_text("face", "Face", face));
                body = body.push(field_text("speaker", "Speaker", speaker));
                for index in 0..4 {
                    let line = lines.get(index).map(String::as_str).unwrap_or("");
                    body = body.push(field_text(
                        &format!("line.{index}"),
                        &format!("Line {}", index + 1),
                        line,
                    ));
                }
            }
            Command::ShowChoices {
                choices, cancel, ..
            } => {
                for (index, choice) in choices.iter().enumerate() {
                    body = body.push(field_text(
                        &format!("choice.{index}"),
                        &format!("Choice {}", index + 1),
                        choice,
                    ));
                }
                body = body.push(
                    row![
                        btn("Add choice", Message::ChoiceAdd),
                        btn("Remove choice", Message::ChoiceRemove)
                    ]
                    .spacing(8),
                );
                body = body.push(field_pick(
                    "cmd.cancel",
                    "Cancel",
                    cancel_options(choices),
                    cancel_current(cancel, choices),
                ));
            }
            Command::Comment { text } => body = body.push(field_text("comment", "Comment", text)),
            Command::ControlSwitch { id, value } => {
                body = body.push(field_num(self, "cmd.switch", "Switch", *id));
                body = body.push(field_pick(
                    "cmd.switch_value",
                    "Value",
                    labels(&SwitchValue::all()),
                    value.to_string(),
                ));
            }
            Command::ControlVariable { id, op, operand } => {
                body = body.push(field_num(self, "cmd.var", "Variable", *id));
                body = body.push(field_pick(
                    "cmd.var_op",
                    "Operation",
                    labels(&VarOp::all()),
                    op.to_string(),
                ));
                body = body.push(self.operand_form("var", operand));
            }
            Command::ControlSelfSwitch { key, on } => {
                body = body.push(field_pick(
                    "cmd.self",
                    "Self switch",
                    labels(&SelfSwitch::all()),
                    key.to_string(),
                ));
                body = body.push(check("cmd.self_on", "On", *on));
            }
            Command::Conditional {
                cond, else_branch, ..
            } => {
                body = body.push(self.condition_form(cond));
                body = body.push(check("cmd.else", "Else branch", else_branch.is_some()));
            }
            Command::CallCommon { id } => {
                body = body.push(field_num(self, "cmd.common", "Common event", *id))
            }
            Command::ExitEvent => {
                body = body.push(text(
                    "Stops this event. Later commands on the page do not run.",
                ))
            }
            Command::Wait { frames } => {
                body = body.push(field_num(self, "cmd.wait", "Frames", *frames))
            }
            Command::TransferPlayer {
                map_id,
                x,
                y,
                direction,
                fade,
            } => {
                body = body.push(
                    row![
                        field_num(self, "cmd.map", "Map", *map_id),
                        field_num(self, "cmd.x", "X", *x),
                        field_num(self, "cmd.y", "Y", *y),
                    ]
                    .spacing(8),
                );
                body = body.push(field_pick(
                    "cmd.dir",
                    "Direction",
                    dir_options(),
                    dir_current(*direction),
                ));
                body = body.push(check("cmd.fade", "Fade", *fade));
            }
            Command::SetMoveRoute { target, route } => {
                body = body.push(field_pick(
                    "cmd.move",
                    "Target",
                    self.move_options(),
                    move_current(self, target),
                ));
                if let MoveTarget::Event { id } = target {
                    body = body.push(field_num(self, "cmd.event", "Event", *id));
                }
                body = body.push(self.route_editor("cmd", route));
            }
            Command::ChangeGold { op, amount } => {
                body = body.push(field_pick(
                    "cmd.gold_op",
                    "Operation",
                    labels(&AmountOp::all()),
                    op.to_string(),
                ));
                body = body.push(field_num(self, "cmd.amount", "Amount", *amount));
            }
            Command::ChangeItems { id, op, amount } => {
                body = body.push(field_num(self, "cmd.item", "Item", *id));
                body = body.push(field_pick(
                    "cmd.item_op",
                    "Operation",
                    labels(&AmountOp::all()),
                    op.to_string(),
                ));
                body = body.push(field_num(self, "cmd.amount", "Amount", *amount));
            }
            Command::ChangeHp { target, op, amount } => {
                body = self.actor_amount(body, "hp", target, op, *amount)
            }
            Command::ChangeMp { target, op, amount } => {
                body = self.actor_amount(body, "mp", target, op, *amount)
            }
            Command::RecoverAll { target } => {
                body = body.push(field_pick(
                    "cmd.recover",
                    "Target",
                    self.actor_options(),
                    actor_current(self, target),
                ))
            }
            Command::GameOver => body = body.push(text("Sends the party to the game over screen.")),
            Command::OpenSave => {
                body = body.push(text(
                    "Opens the save screen. The rest of the event continues after it closes.",
                ))
            }
            Command::OpenMenu => {
                body = body.push(text(
                    "Opens the party menu. The event continues after it closes.",
                ))
            }
            Command::Battle {
                troop_id,
                can_escape,
                ..
            } => {
                body = body.push(field_num(self, "cmd.troop", "Troop", *troop_id));
                body = body.push(check("cmd.escape", "Can escape", *can_escape));
                body = body.push(text("Win and Escape in the list are the branches."));
            }
            Command::ReturnToTitle => body = body.push(text("Returns to the title screen.")),
            Command::ShowPicture { id, name, x, y } => {
                body = body.push(field_num(self, "cmd.picture", "Picture", *id));
                body = body.push(field_text("picture.name", "Name", name));
                body = body.push(btn("Import picture", Message::Import("pictures".into())));
                body = body.push(
                    row![
                        field_num(self, "cmd.px", "X", *x),
                        field_num(self, "cmd.py", "Y", *y)
                    ]
                    .spacing(8),
                );
            }
            Command::MovePicture { id, x, y, frames } => {
                body = body.push(field_num(self, "cmd.picture", "Picture", *id));
                body = body.push(
                    row![
                        field_num(self, "cmd.px", "X", *x),
                        field_num(self, "cmd.py", "Y", *y),
                        field_num(self, "cmd.frames", "Frames", *frames),
                    ]
                    .spacing(8),
                );
            }
            Command::ErasePicture { id } => {
                body = body.push(field_num(self, "cmd.picture", "Picture", *id))
            }
        }
        body.into()
    }

    fn condition_form(&self, cond: &Condition) -> Element<'_, Message> {
        let mut body = Column::new().spacing(6);
        body = body.push(field_pick(
            "cond.kind",
            "Condition",
            cond_kinds(),
            cond_kind(cond),
        ));
        match cond {
            Condition::Switch { id, on } => {
                body = body.push(field_num(self, "cond.switch", "Switch", *id));
                body = body.push(check("cond.on", "Is on", *on));
            }
            Condition::Variable { id, cmp, operand } => {
                body = body.push(field_num(self, "cond.var", "Variable", *id));
                body = body.push(field_pick(
                    "cond.cmp",
                    "Compare",
                    labels(&Cmp::all()),
                    cmp.to_string(),
                ));
                body = body.push(self.operand_form("cond", operand));
            }
            Condition::SelfSwitch { key, on } => {
                body = body.push(field_pick(
                    "cond.self",
                    "Self switch",
                    labels(&SelfSwitch::all()),
                    key.to_string(),
                ));
                body = body.push(check("cond.on", "Is on", *on));
            }
            Condition::Item { id } => body = body.push(field_num(self, "cond.item", "Item", *id)),
            Condition::Gold { cmp, amount } => {
                body = body.push(field_pick(
                    "cond.cmp",
                    "Compare",
                    labels(&Cmp::all()),
                    cmp.to_string(),
                ));
                body = body.push(field_num(self, "cond.gold", "Gold", *amount));
            }
            Condition::ActorInParty { id } => {
                body = body.push(field_num(self, "cond.actor", "Actor", *id))
            }
            Condition::ActorHp { id, cmp, amount } => {
                body = body.push(field_num(self, "cond.hp", "Actor", *id));
                body = body.push(field_pick(
                    "cond.cmp",
                    "Compare",
                    labels(&Cmp::all()),
                    cmp.to_string(),
                ));
                body = body.push(field_num(self, "cond.amount", "HP", *amount));
            }
        }
        body.into()
    }

    fn operand_form(&self, prefix: &str, operand: &Operand) -> Element<'_, Message> {
        let mut body = Column::new().spacing(6);
        body = body.push(field_pick(
            &format!("{prefix}.operand"),
            "Operand",
            operand_kinds(),
            operand_kind(operand),
        ));
        match operand {
            Operand::Constant { value } => {
                body = body.push(field_num(self, &format!("{prefix}.value"), "Value", *value))
            }
            Operand::Variable { id }
            | Operand::ItemCount { id }
            | Operand::ActorHp { id }
            | Operand::ActorMp { id }
            | Operand::ActorLevel { id } => {
                body = body.push(field_num(self, &format!("{prefix}.id"), "Id", *id));
            }
            Operand::Random { min, max } => {
                body = body.push(
                    row![
                        field_num(self, &format!("{prefix}.min"), "Min", *min),
                        field_num(self, &format!("{prefix}.max"), "Max", *max)
                    ]
                    .spacing(8),
                );
            }
            Operand::Gold => {}
        }
        body.into()
    }

    fn route_editor(&self, owner: &str, route: &MoveRoute) -> Element<'_, Message> {
        let mut body = Column::new().spacing(4);
        for (index, step) in route.steps.iter().enumerate() {
            let mut line = row![text(step.to_string()).width(Length::Fixed(220.0))].spacing(6);
            match step {
                MoveStep::Wait { frames } => {
                    line = line.push(field_num(
                        self,
                        &format!("route.{owner}.wait.{index}"),
                        "Frames",
                        *frames,
                    ));
                }
                MoveStep::Speed { speed } => {
                    line = line.push(field_num(
                        self,
                        &format!("route.{owner}.speed.{index}"),
                        "Speed",
                        *speed,
                    ));
                }
                MoveStep::Through { on } => {
                    line = line.push(check(
                        &format!("route.{owner}.through.{index}"),
                        "Through",
                        *on,
                    ));
                }
                _ => {}
            }
            line = line.push(button(text("Remove")).on_press(Message::Route {
                owner: owner.into(),
                op: "del".into(),
                index,
            }));
            body = body.push(line);
        }
        let catalog: Vec<String> = MoveStep::catalog()
            .iter()
            .map(|step| step.to_string())
            .collect();
        let owner_owned = owner.to_string();
        body = body.push(pick_list(catalog, None::<String>, move |value| {
            let index = MoveStep::catalog()
                .iter()
                .position(|step| step.to_string() == value)
                .unwrap_or(0);
            Message::Route {
                owner: owner_owned.clone(),
                op: "add".into(),
                index,
            }
        }));
        body = body.push(
            row![
                check(&format!("route.{owner}.repeat"), "Repeat", route.repeat),
                check(
                    &format!("route.{owner}.skip"),
                    "Skip if blocked",
                    route.skippable
                ),
                check(
                    &format!("route.{owner}.wait"),
                    "Wait for completion",
                    route.wait
                ),
            ]
            .spacing(8),
        );
        body.into()
    }

    fn usage_form(
        &self,
        title: &str,
        id: u32,
        name: &str,
        description: &str,
        note: &str,
        prefix: &str,
        occasion: Occasion,
        scope: Scope,
        damage: &Damage,
        hit: i32,
        cost: i32,
        cost_label: &str,
    ) -> Column<'_, Message> {
        let mut body = Column::new().spacing(8);
        body = body.push(heading(&format!("{title} {id}")));
        body = body.push(field_text(&format!("{prefix}.name"), "Name", name));
        body = body.push(field_text(
            &format!("{prefix}.description"),
            "Description",
            description,
        ));
        body = body.push(field_num(
            self,
            &format!(
                "{prefix}.{}",
                if prefix == "skill" { "mp" } else { "price" }
            ),
            cost_label,
            cost,
        ));
        body = body.push(field_pick(
            &format!("{prefix}.occasion"),
            "Occasion",
            labels(&Occasion::all()),
            occasion.to_string(),
        ));
        body = body.push(field_pick(
            &format!("{prefix}.scope"),
            "Scope",
            labels(&Scope::all()),
            scope.to_string(),
        ));
        body = body.push(field_num(self, &format!("{prefix}.hit"), "Hit %", hit));
        body = body.push(field_pick(
            &format!("{prefix}.kind"),
            "Damage",
            labels(&DamageKind::all()),
            damage.kind.to_string(),
        ));
        body = body.push(field_pick(
            &format!("{prefix}.element"),
            "Element",
            labels(&fablespace_data::Element::all()),
            damage.element.to_string(),
        ));
        body = body.push(
            row![
                field_num(self, &format!("{prefix}.power"), "Power", damage.power),
                field_num(
                    self,
                    &format!("{prefix}.variance"),
                    "Variance %",
                    damage.variance
                ),
            ]
            .spacing(8),
        );
        body = body.push(text("Physical damage is power + attack x 4 - defense x 2. Magical damage uses magic attack and magic defense. Magical recovery adds magic attack x 2.").size(14));
        body = body.push(field_text(&format!("{prefix}.note"), "Note", note));
        body
    }

    fn actor_amount<'a>(
        &'a self,
        mut body: Column<'a, Message>,
        kind: &str,
        target: &ActorTarget,
        op: &AmountOp,
        amount: i32,
    ) -> Column<'a, Message> {
        body = body.push(field_pick(
            &format!("cmd.{kind}_target"),
            "Target",
            self.actor_options(),
            actor_current(self, target),
        ));
        if let ActorTarget::Actor(id) = target {
            body = body.push(field_num(self, "cmd.actor", "Actor", *id));
        }
        body = body.push(field_pick(
            &format!("cmd.{kind}_op"),
            "Operation",
            labels(&AmountOp::all()),
            op.to_string(),
        ));
        body = body.push(field_num(self, "cmd.amount", "Amount", amount));
        body
    }

    fn actor_options(&self) -> Vec<String> {
        let mut options = vec!["Entire Party".into()];
        for actor in self.game.actors.iter().filter(|actor| actor.id != 0) {
            options.push(format!("{} {}", actor.id, actor.name));
        }
        options
    }

    fn move_options(&self) -> Vec<String> {
        let mut options = vec!["Player".into(), "This Event".into()];
        if let Some(map) = self.map() {
            for event in &map.events {
                options.push(format!("Event {} {}", event.id, event.name));
            }
        }
        options
    }

    fn selected_command(&self) -> Option<Command> {
        let mut list = self.command_slice()?.to_vec();
        let Target::Command(path) = self.cmd_target.clone()? else {
            return None;
        };
        command::command_mut(&mut list, &path).cloned()
    }

    fn lookup_name(&self, id: u32, kind: &str) -> String {
        if id == 0 {
            return "None".into();
        }
        match kind {
            "class" => self.game.class(id).map(|c| c.name.clone()),
            "weapon" => self.game.weapon(id).map(|c| c.name.clone()),
            "armor" => self.game.armor(id).map(|c| c.name.clone()),
            _ => None,
        }
        .unwrap_or_else(|| format!("{id}"))
    }

    fn actor_mut(&mut self) -> Option<&mut Actor> {
        slot(&mut self.game.actors, self.sel)
    }
    fn class_mut(&mut self) -> Option<&mut Class> {
        slot(&mut self.game.classes, self.sel)
    }
    fn skill_mut(&mut self) -> Option<&mut Skill> {
        slot(&mut self.game.skills, self.sel)
    }
    fn item_mut(&mut self) -> Option<&mut Item> {
        slot(&mut self.game.items, self.sel)
    }
    fn weapon_mut(&mut self) -> Option<&mut Weapon> {
        slot(&mut self.game.weapons, self.sel)
    }
    fn armor_mut(&mut self) -> Option<&mut Armor> {
        slot(&mut self.game.armors, self.sel)
    }
    fn enemy_mut(&mut self) -> Option<&mut Enemy> {
        slot(&mut self.game.enemies, self.sel)
    }
    fn troop_mut(&mut self) -> Option<&mut Troop> {
        slot(&mut self.game.troops, self.sel)
    }
    fn state_mut(&mut self) -> Option<&mut State> {
        slot(&mut self.game.states, self.sel)
    }
    fn tileset_mut(&mut self) -> Option<&mut Tileset> {
        slot(&mut self.game.tilesets, self.sel)
    }
    fn common_mut(&mut self) -> Option<&mut CommonEvent> {
        slot(&mut self.game.common_events, self.sel)
    }
    fn troop_page_mut(&mut self) -> Option<&mut TroopPage> {
        let page = self.page;
        self.troop_mut()?.pages.get_mut(page)
    }

    fn set_growth(&mut self, rest: &str, base: bool) -> bool {
        let Some(index) = rest.parse::<usize>().ok() else {
            return false;
        };
        let Some(value) = self.num_buf.parse::<i32>().ok() else {
            return false;
        };
        let Some(class) = self.class_mut() else {
            return false;
        };
        while class.params.len() <= index {
            class.params.push(ParamGrowth::default());
        }
        if base {
            class.params[index].base = value;
        } else {
            class.params[index].plus = value;
        }
        true
    }

    fn set_weapon_param(&mut self, rest: &str) -> bool {
        self.set_param_list(rest, true)
    }
    fn set_armor_param(&mut self, rest: &str) -> bool {
        self.set_param_list(rest, false)
    }

    fn set_param_list(&mut self, rest: &str, weapon: bool) -> bool {
        let Some(index) = rest.parse::<usize>().ok() else {
            return false;
        };
        let Some(value) = self.num_buf.parse::<i32>().ok() else {
            return false;
        };
        let params = if weapon {
            self.weapon_mut().map(|item| &mut item.params)
        } else {
            self.armor_mut().map(|item| &mut item.params)
        };
        let Some(params) = params else {
            return false;
        };
        if params.len() < PARAM_COUNT {
            params.resize(PARAM_COUNT, 0);
        }
        if index < params.len() {
            params[index] = value;
            true
        } else {
            false
        }
    }

    fn set_enemy_param(&mut self, rest: &str) -> bool {
        let Some(index) = rest.parse::<usize>().ok() else {
            return false;
        };
        let Some(value) = self.num_buf.parse::<i32>().ok() else {
            return false;
        };
        let Some(enemy) = self.enemy_mut() else {
            return false;
        };
        if enemy.params.len() < PARAM_COUNT {
            enemy.params.resize(PARAM_COUNT, 0);
        }
        if index < enemy.params.len() {
            enemy.params[index] = value.max(0);
            true
        } else {
            false
        }
    }

    fn set_stack_num(&mut self, rest: &str) -> bool {
        let mut parts = rest.split('.');
        let kind = parts.next().unwrap_or("");
        let field = parts.next().unwrap_or("");
        let Some(index) = parts.next().and_then(|part| part.parse::<usize>().ok()) else {
            return false;
        };
        let Some(value) = self.num_buf.parse::<u32>().ok() else {
            return false;
        };
        let list = match kind {
            "item" => &mut self.game.system.start_items,
            "weapon" => &mut self.game.system.start_weapons,
            "armor" => &mut self.game.system.start_armors,
            _ => return false,
        };
        let Some(stack) = list.get_mut(index) else {
            return false;
        };
        match field {
            "id" => stack.id = value,
            "count" => stack.count = value.max(1),
            _ => return false,
        }
        true
    }

    fn set_effect_num(&mut self, rest: &str) -> bool {
        let mut parts = rest.split('.');
        let owner = parts.next().unwrap_or("");
        let field = parts.next().unwrap_or("");
        let Some(index) = parts.next().and_then(|part| part.parse::<usize>().ok()) else {
            return false;
        };
        let Some(value) = self.num_buf.parse::<i32>().ok() else {
            return false;
        };
        let effects = match owner {
            "skill" => self.skill_mut().map(|skill| &mut skill.effects),
            "item" => self.item_mut().map(|item| &mut item.effects),
            _ => None,
        };
        let Some(effect) = effects.and_then(|effects| effects.get_mut(index)) else {
            return false;
        };
        match (field, effect) {
            ("percent", Effect::RecoverHp { percent, .. } | Effect::RecoverMp { percent, .. }) => {
                *percent = value
            }
            ("flat", Effect::RecoverHp { flat, .. } | Effect::RecoverMp { flat, .. }) => {
                *flat = value
            }
            ("state", Effect::AddState { state_id, .. } | Effect::RemoveState { state_id }) => {
                *state_id = value.max(0) as u32
            }
            ("chance", Effect::AddState { chance, .. }) => *chance = value.clamp(0, 100),
            _ => return false,
        }
        true
    }

    fn set_learning_num(&mut self, rest: &str) -> bool {
        let mut parts = rest.split('.');
        let field = parts.next().unwrap_or("");
        let Some(index) = parts.next().and_then(|part| part.parse::<usize>().ok()) else {
            return false;
        };
        let Some(value) = self.num_buf.parse::<u32>().ok() else {
            return false;
        };
        let Some(learning) = self
            .class_mut()
            .and_then(|class| class.learnings.get_mut(index))
        else {
            return false;
        };
        match field {
            "level" => learning.level = value.max(1),
            "skill" => learning.skill_id = value,
            _ => return false,
        }
        true
    }

    fn set_drop_num(&mut self, rest: &str) -> bool {
        let mut parts = rest.split('.');
        let field = parts.next().unwrap_or("");
        let Some(index) = parts.next().and_then(|part| part.parse::<usize>().ok()) else {
            return false;
        };
        let Some(value) = self.num_buf.parse::<u32>().ok() else {
            return false;
        };
        let Some(drop) = self
            .enemy_mut()
            .and_then(|enemy| enemy.drops.get_mut(index))
        else {
            return false;
        };
        match field {
            "item" => drop.item_id = value,
            "den" => drop.denominator = value.max(1),
            _ => return false,
        }
        true
    }

    fn set_action_num(&mut self, rest: &str) -> bool {
        let mut parts = rest.split('.');
        let field = parts.next().unwrap_or("");
        let Some(index) = parts.next().and_then(|part| part.parse::<usize>().ok()) else {
            return false;
        };
        let skill: u32 = self.num_buf.parse().unwrap_or(0);
        let rating = self.num_buf.parse::<u32>().unwrap_or(1).max(1);
        let hp = self.num_buf.parse::<i32>().unwrap_or(50).clamp(1, 100);
        let turn = self.num_buf.parse().unwrap_or(0);
        let every = self.num_buf.parse().unwrap_or(1);
        let Some(action) = self
            .enemy_mut()
            .and_then(|enemy| enemy.actions.get_mut(index))
        else {
            return false;
        };
        match field {
            "skill" => action.skill_id = skill,
            "rating" => action.rating = rating,
            "hp" => {
                if let ActionCondition::HpBelow { percent } = &mut action.condition {
                    *percent = hp;
                }
            }
            "turn" => {
                if let ActionCondition::Turn { start, .. } = &mut action.condition {
                    *start = turn;
                }
            }
            "every" => {
                if let ActionCondition::Turn { every: slot, .. } = &mut action.condition {
                    *slot = every;
                }
            }
            _ => return false,
        }
        true
    }

    fn set_member_num(&mut self, rest: &str) -> bool {
        let mut parts = rest.split('.');
        let field = parts.next().unwrap_or("");
        let Some(index) = parts.next().and_then(|part| part.parse::<usize>().ok()) else {
            return false;
        };
        let enemy_id = self.num_buf.parse().unwrap_or(1);
        let coord = self.num_buf.parse().unwrap_or(0);
        let Some(member) = self
            .troop_mut()
            .and_then(|troop| troop.members.get_mut(index))
        else {
            return false;
        };
        match field {
            "enemy" => member.enemy_id = enemy_id,
            "x" => member.x = coord,
            "y" => member.y = coord,
            _ => return false,
        }
        true
    }

    fn set_autotile_num(&mut self, rest: &str) -> bool {
        let mut parts = rest.split('.');
        let field = parts.next().unwrap_or("");
        let Some(index) = parts.next().and_then(|part| part.parse::<usize>().ok()) else {
            return false;
        };
        let Some(value) = self.num_buf.parse::<u32>().ok() else {
            return false;
        };
        let Some(auto) = self
            .tileset_mut()
            .and_then(|tileset| tileset.autotiles.get_mut(index))
        else {
            return false;
        };
        match field {
            "tile" => auto.tile_id = value,
            "col" => auto.origin_col = value,
            "row" => auto.origin_row = value,
            _ => return false,
        }
        true
    }

    fn set_route_num(&mut self, rest: &str) -> bool {
        let mut parts = rest.split('.');
        let owner = parts.next().unwrap_or("").to_string();
        let field = parts.next().unwrap_or("").to_string();
        let Some(index) = parts.next().and_then(|part| part.parse::<usize>().ok()) else {
            return false;
        };
        let Some(value) = self.num_buf.parse::<u32>().ok() else {
            return false;
        };
        let mut wrote = false;
        self.with_route(&owner, |route| {
            if let Some(step) = route.steps.get_mut(index) {
                match (field.as_str(), step) {
                    ("wait", MoveStep::Wait { frames }) => {
                        *frames = value;
                        wrote = true;
                    }
                    ("speed", MoveStep::Speed { speed }) => {
                        *speed = value.clamp(1, 5) as u8;
                        wrote = true;
                    }
                    _ => {}
                }
            }
        });
        wrote
    }

    fn set_state_turns(&mut self, min_field: bool, value: u32) -> bool {
        let Some(state) = self.state_mut() else {
            return false;
        };
        if let Removal::Turns { min, max } = &mut state.removal {
            if min_field {
                *min = value.max(1);
            } else {
                *max = value.max(*min);
            }
            true
        } else {
            false
        }
    }

    fn set_troop_hp(&mut self, percent: i32) -> bool {
        let Some(page) = self.troop_page_mut() else {
            return false;
        };
        if percent <= 0 {
            page.condition.enemy_hp = None;
        } else if let Some((_, slot)) = page.condition.enemy_hp.as_mut() {
            *slot = percent.clamp(1, 100);
        } else {
            page.condition.enemy_hp = Some((0, percent.clamp(1, 100)));
        }
        true
    }

    fn set_troop_member(&mut self, index: usize) -> bool {
        let Some(page) = self.troop_page_mut() else {
            return false;
        };
        if let Some((slot, _)) = page.condition.enemy_hp.as_mut() {
            *slot = index;
        } else if index > 0 {
            page.condition.enemy_hp = Some((index, 50));
        }
        true
    }

    fn set_cmd_u32(&mut self, field: &str, value: u32) -> bool {
        let mut wrote = false;
        self.update_cmd(|cmd| {
            wrote = assign_u32(cmd, field, value);
            wrote
        });
        wrote
    }

    fn set_cmd_i32(&mut self, field: &str, value: i32) -> bool {
        let mut wrote = false;
        self.update_cmd(|cmd| {
            wrote = assign_i32(cmd, field, value);
            wrote
        });
        wrote
    }

    fn set_cond_num(&mut self, field: &str, signed: i32, unsigned: u32) -> bool {
        let mut wrote = false;
        self.update_cmd(|cmd| {
            let Command::Conditional { cond, .. } = cmd else {
                return false;
            };
            wrote = match (field, cond) {
                ("switch", Condition::Switch { id, .. }) => {
                    *id = unsigned;
                    true
                }
                ("var", Condition::Variable { id, .. }) => {
                    *id = unsigned;
                    true
                }
                ("item", Condition::Item { id }) => {
                    *id = unsigned;
                    true
                }
                ("gold", Condition::Gold { amount, .. }) => {
                    *amount = signed;
                    true
                }
                ("actor", Condition::ActorInParty { id }) => {
                    *id = unsigned;
                    true
                }
                ("hp", Condition::ActorHp { id, .. }) => {
                    *id = unsigned;
                    true
                }
                ("amount", Condition::ActorHp { amount, .. }) => {
                    *amount = signed;
                    true
                }
                _ => false,
            };
            wrote
        });
        wrote
    }

    fn set_operand_num(
        &mut self,
        condition: bool,
        field: &str,
        signed: i32,
        unsigned: u32,
    ) -> bool {
        let mut wrote = false;
        self.update_cmd(|cmd| {
            let Some(operand) = operand_mut(cmd, condition) else {
                return false;
            };
            wrote = match (field, operand) {
                ("value", Operand::Constant { value }) => {
                    *value = signed;
                    true
                }
                (
                    "id",
                    Operand::Variable { id }
                    | Operand::ItemCount { id }
                    | Operand::ActorHp { id }
                    | Operand::ActorMp { id }
                    | Operand::ActorLevel { id },
                ) => {
                    *id = unsigned;
                    true
                }
                ("min", Operand::Random { min, .. }) => {
                    *min = signed;
                    true
                }
                ("max", Operand::Random { max, .. }) => {
                    *max = signed;
                    true
                }
                _ => false,
            };
            wrote
        });
        wrote
    }

    fn set_effect_kind(&mut self, rest: &str, value: &str) -> bool {
        let mut parts = rest.split('.');
        let owner = parts.next().unwrap_or("");
        let Some(index) = parts.next().and_then(|part| part.parse::<usize>().ok()) else {
            return false;
        };
        let next = effect_from_kind(value);
        let effects = match owner {
            "skill" => self.skill_mut().map(|skill| &mut skill.effects),
            "item" => self.item_mut().map(|item| &mut item.effects),
            _ => None,
        };
        if let Some(slot) = effects.and_then(|effects| effects.get_mut(index)) {
            *slot = next;
            true
        } else {
            false
        }
    }

    fn set_action_cond(&mut self, rest: &str, value: &str) -> bool {
        let Some(index) = rest.parse::<usize>().ok() else {
            return false;
        };
        let Some(action) = self
            .enemy_mut()
            .and_then(|enemy| enemy.actions.get_mut(index))
        else {
            return false;
        };
        action.condition = match value {
            "HP below" => ActionCondition::HpBelow { percent: 50 },
            "Turn" => ActionCondition::Turn { start: 0, every: 1 },
            _ => ActionCondition::Always,
        };
        true
    }

    fn set_removal(&mut self, value: &str) -> bool {
        let Some(state) = self.state_mut() else {
            return false;
        };
        state.removal = match value {
            "After turns" => Removal::Turns { min: 3, max: 3 },
            "On damage" => Removal::OnDamage,
            _ => Removal::None,
        };
        true
    }

    fn set_page_self(&mut self, value: &str) -> bool {
        let parsed = if value == "None" {
            None
        } else {
            by_label(&SelfSwitch::all(), value)
        };
        if value != "None" && parsed.is_none() {
            return false;
        }
        self.page_mut()
            .map(|page| page.conditions.self_switch = parsed)
            .is_some()
    }

    fn set_switch_value(&mut self, value: &str) -> bool {
        let Some(parsed) = by_label(&SwitchValue::all(), value) else {
            return false;
        };
        let mut wrote = false;
        self.update_cmd(|cmd| {
            if let Command::ControlSwitch { value, .. } = cmd {
                *value = parsed;
                wrote = true;
            }
            wrote
        });
        wrote
    }

    fn set_var_op(&mut self, value: &str) -> bool {
        let Some(parsed) = by_label(&VarOp::all(), value) else {
            return false;
        };
        let mut wrote = false;
        self.update_cmd(|cmd| {
            if let Command::ControlVariable { op, .. } = cmd {
                *op = parsed;
                wrote = true;
            }
            wrote
        });
        wrote
    }

    fn set_self_key(&mut self, value: &str) -> bool {
        let Some(parsed) = by_label(&SelfSwitch::all(), value) else {
            return false;
        };
        let mut wrote = false;
        self.update_cmd(|cmd| {
            if let Command::ControlSelfSwitch { key, .. } = cmd {
                *key = parsed;
                wrote = true;
            }
            wrote
        });
        wrote
    }

    fn set_transfer_dir(&mut self, value: &str) -> bool {
        let parsed = if value == "Unchanged" {
            None
        } else {
            by_label(&Direction::all(), value)
        };
        if value != "Unchanged" && parsed.is_none() {
            return false;
        }
        let mut wrote = false;
        self.update_cmd(|cmd| {
            if let Command::TransferPlayer { direction, .. } = cmd {
                *direction = parsed;
                wrote = true;
            }
            wrote
        });
        wrote
    }

    fn set_amount_op(&mut self, key: &str, value: &str) -> bool {
        let Some(parsed) = by_label(&AmountOp::all(), value) else {
            return false;
        };
        let mut wrote = false;
        self.update_cmd(|cmd| {
            let slot = match (key, cmd) {
                ("cmd.gold_op", Command::ChangeGold { op, .. }) => Some(op),
                ("cmd.item_op", Command::ChangeItems { op, .. }) => Some(op),
                ("cmd.hp_op", Command::ChangeHp { op, .. }) => Some(op),
                ("cmd.mp_op", Command::ChangeMp { op, .. }) => Some(op),
                _ => None,
            };
            if let Some(slot) = slot {
                *slot = parsed;
                wrote = true;
            }
            wrote
        });
        wrote
    }

    fn set_actor_target(&mut self, key: &str, value: &str) -> bool {
        let parsed = actor_from_label(value);
        let mut wrote = false;
        self.update_cmd(|cmd| {
            let slot = match (key, cmd) {
                ("cmd.hp_target", Command::ChangeHp { target, .. }) => Some(target),
                ("cmd.mp_target", Command::ChangeMp { target, .. }) => Some(target),
                ("cmd.recover", Command::RecoverAll { target }) => Some(target),
                _ => None,
            };
            if let Some(slot) = slot {
                *slot = parsed;
                wrote = true;
            }
            wrote
        });
        wrote
    }

    fn set_move_target(&mut self, value: &str) -> bool {
        let parsed = if value == "Player" {
            MoveTarget::Player
        } else if value == "This Event" {
            MoveTarget::ThisEvent
        } else if let Some(id) = value
            .strip_prefix("Event ")
            .and_then(|rest| leading_id(rest))
        {
            MoveTarget::Event { id }
        } else {
            return false;
        };
        let mut wrote = false;
        self.update_cmd(|cmd| {
            if let Command::SetMoveRoute { target, .. } = cmd {
                *target = parsed;
                wrote = true;
            }
            wrote
        });
        wrote
    }

    fn set_cancel(&mut self, value: &str) -> bool {
        let mut wrote = false;
        self.update_cmd(|cmd| {
            let Command::ShowChoices {
                choices, cancel, ..
            } = cmd
            else {
                return false;
            };
            *cancel = if value == "Disallow" {
                CancelMode::Disallow
            } else if value == "Branch" {
                CancelMode::Branch
            } else if let Some(index) = value
                .strip_prefix("Choice ")
                .and_then(|rest| rest.split(':').next())
                .and_then(|n| n.trim().parse::<usize>().ok())
            {
                if index < choices.len() {
                    CancelMode::Choice(index)
                } else {
                    CancelMode::Disallow
                }
            } else {
                return false;
            };
            wrote = true;
            true
        });
        wrote
    }

    fn set_cond_kind(&mut self, value: &str) -> bool {
        let next = match value {
            "Variable" => Condition::Variable {
                id: 1,
                cmp: Cmp::Ge,
                operand: Operand::Constant { value: 1 },
            },
            "Self switch" => Condition::SelfSwitch {
                key: SelfSwitch::A,
                on: true,
            },
            "Item" => Condition::Item { id: 1 },
            "Gold" => Condition::Gold {
                cmp: Cmp::Ge,
                amount: 1,
            },
            "Actor in party" => Condition::ActorInParty { id: 1 },
            "Actor HP" => Condition::ActorHp {
                id: 1,
                cmp: Cmp::Le,
                amount: 10,
            },
            _ => Condition::Switch { id: 1, on: true },
        };
        let mut wrote = false;
        self.update_cmd(|cmd| {
            if let Command::Conditional { cond, .. } = cmd {
                *cond = next.clone();
                wrote = true;
            }
            wrote
        });
        wrote
    }

    fn set_cond_cmp(&mut self, value: &str) -> bool {
        let Some(parsed) = by_label(&Cmp::all(), value) else {
            return false;
        };
        let mut wrote = false;
        self.update_cmd(|cmd| {
            if let Command::Conditional { cond, .. } = cmd {
                match cond {
                    Condition::Variable { cmp, .. }
                    | Condition::Gold { cmp, .. }
                    | Condition::ActorHp { cmp, .. } => {
                        *cmp = parsed;
                        wrote = true;
                    }
                    _ => {}
                }
            }
            wrote
        });
        wrote
    }

    fn set_cond_self(&mut self, value: &str) -> bool {
        let Some(parsed) = by_label(&SelfSwitch::all(), value) else {
            return false;
        };
        let mut wrote = false;
        self.update_cmd(|cmd| {
            if let Command::Conditional {
                cond: Condition::SelfSwitch { key, .. },
                ..
            } = cmd
            {
                *key = parsed;
                wrote = true;
            }
            wrote
        });
        wrote
    }

    fn set_cond_on(&mut self, on: bool) -> bool {
        let mut wrote = false;
        self.update_cmd(|cmd| {
            if let Command::Conditional { cond, .. } = cmd {
                match cond {
                    Condition::Switch { on: slot, .. } | Condition::SelfSwitch { on: slot, .. } => {
                        *slot = on;
                        wrote = true;
                    }
                    _ => {}
                }
            }
            wrote
        });
        wrote
    }

    fn set_operand_kind(&mut self, condition: bool, value: &str) -> bool {
        let next = match value {
            "Variable" => Operand::Variable { id: 1 },
            "Random" => Operand::Random { min: 0, max: 1 },
            "Gold" => Operand::Gold,
            "Item count" => Operand::ItemCount { id: 1 },
            "Actor HP" => Operand::ActorHp { id: 1 },
            "Actor MP" => Operand::ActorMp { id: 1 },
            "Actor level" => Operand::ActorLevel { id: 1 },
            _ => Operand::Constant { value: 0 },
        };
        let mut wrote = false;
        self.update_cmd(|cmd| {
            if let Some(operand) = operand_mut(cmd, condition) {
                *operand = next.clone();
                wrote = true;
            }
            wrote
        });
        wrote
    }

    fn set_route_toggle(&mut self, rest: &str, on: bool) -> bool {
        let mut parts = rest.split('.');
        let owner = parts.next().unwrap_or("").to_string();
        let field = parts.next().unwrap_or("").to_string();
        let index = parts.next().and_then(|part| part.parse::<usize>().ok());
        let mut wrote = false;
        self.with_route(&owner, |route| match field.as_str() {
            "repeat" => {
                route.repeat = on;
                wrote = true;
            }
            "skip" => {
                route.skippable = on;
                wrote = true;
            }
            "wait" => {
                route.wait = on;
                wrote = true;
            }
            "through" => {
                if let Some(index) = index {
                    if let Some(MoveStep::Through { on: slot }) = route.steps.get_mut(index) {
                        *slot = on;
                        wrote = true;
                    }
                }
            }
            _ => {}
        });
        wrote
    }

    fn with_route(&mut self, owner: &str, f: impl FnOnce(&mut MoveRoute)) {
        if owner == "page" {
            if let Some(page) = self.page_mut() {
                f(&mut page.route);
                self.changed();
            }
            return;
        }
        let Some(Target::Command(path)) = self.cmd_target.clone() else {
            return;
        };
        let applied = {
            let Some(list) = self.commands_mut() else {
                return;
            };
            let Some(cmd) = command::command_mut(list, &path) else {
                return;
            };
            if let Command::SetMoveRoute { route, .. } = cmd {
                f(route);
                true
            } else {
                false
            }
        };
        if applied {
            self.changed();
        }
    }
}

fn wrote_term(terms: &mut Terms, field: &str, value: String) {
    let slot = match field {
        "hp" => &mut terms.hp,
        "mp" => &mut terms.mp,
        "level" => &mut terms.level,
        "attack" => &mut terms.attack,
        "guard" => &mut terms.guard,
        "item" => &mut terms.item,
        "skill" => &mut terms.skill,
        "equip" => &mut terms.equip,
        "status" => &mut terms.status,
        "save" => &mut terms.save,
        "game_end" => &mut terms.game_end,
        "new_game" => &mut terms.new_game,
        "continue_game" => &mut terms.continue_game,
        "shutdown" => &mut terms.shutdown,
        "fight" => &mut terms.fight,
        "escape" => &mut terms.escape,
        "to_title" => &mut terms.to_title,
        "gold" => &mut terms.gold,
        _ => return,
    };
    *slot = value;
}

fn term_fields(terms: &Terms) -> [(&str, &str, &str); 18] {
    [
        ("system.term.hp", "HP", &terms.hp),
        ("system.term.mp", "MP", &terms.mp),
        ("system.term.level", "Level", &terms.level),
        ("system.term.attack", "Attack", &terms.attack),
        ("system.term.guard", "Guard", &terms.guard),
        ("system.term.item", "Item", &terms.item),
        ("system.term.skill", "Skill", &terms.skill),
        ("system.term.equip", "Equip", &terms.equip),
        ("system.term.status", "Status", &terms.status),
        ("system.term.save", "Save", &terms.save),
        ("system.term.game_end", "End game", &terms.game_end),
        ("system.term.new_game", "New game", &terms.new_game),
        (
            "system.term.continue_game",
            "Continue",
            &terms.continue_game,
        ),
        ("system.term.shutdown", "Exit", &terms.shutdown),
        ("system.term.fight", "Fight", &terms.fight),
        ("system.term.escape", "Escape", &terms.escape),
        ("system.term.to_title", "To title", &terms.to_title),
        ("system.term.gold", "Gold", &terms.gold),
    ]
}

fn assign_u32(cmd: &mut Command, field: &str, value: u32) -> bool {
    match (field, cmd) {
        ("switch", Command::ControlSwitch { id, .. }) => *id = value,
        ("var", Command::ControlVariable { id, .. }) => *id = value,
        ("common", Command::CallCommon { id }) => *id = value,
        ("wait", Command::Wait { frames }) => *frames = value,
        ("map", Command::TransferPlayer { map_id, .. }) => *map_id = value,
        ("x", Command::TransferPlayer { x, .. }) => *x = value,
        ("y", Command::TransferPlayer { y, .. }) => *y = value,
        ("item", Command::ChangeItems { id, .. }) => *id = value,
        ("actor", Command::ChangeHp { target, .. } | Command::ChangeMp { target, .. }) => {
            *target = ActorTarget::Actor(value)
        }
        ("event", Command::SetMoveRoute { target, .. }) => {
            *target = MoveTarget::Event { id: value }
        }
        (
            "picture",
            Command::ShowPicture { id, .. }
            | Command::MovePicture { id, .. }
            | Command::ErasePicture { id },
        ) => *id = value,
        ("frames", Command::MovePicture { frames, .. }) => *frames = value.max(1),
        ("troop", Command::Battle { troop_id, .. }) => *troop_id = value,
        _ => return false,
    }
    true
}

fn assign_i32(cmd: &mut Command, field: &str, value: i32) -> bool {
    match (field, cmd) {
        (
            "amount",
            Command::ChangeGold { amount, .. }
            | Command::ChangeItems { amount, .. }
            | Command::ChangeHp { amount, .. }
            | Command::ChangeMp { amount, .. },
        ) => *amount = value,
        ("px", Command::ShowPicture { x, .. } | Command::MovePicture { x, .. }) => *x = value,
        ("py", Command::ShowPicture { y, .. } | Command::MovePicture { y, .. }) => *y = value,
        _ => return false,
    }
    true
}

fn operand_mut(cmd: &mut Command, condition: bool) -> Option<&mut Operand> {
    if condition {
        if let Command::Conditional {
            cond: Condition::Variable { operand, .. },
            ..
        } = cmd
        {
            return Some(operand);
        }
        None
    } else if let Command::ControlVariable { operand, .. } = cmd {
        Some(operand)
    } else {
        None
    }
}

fn slot<T>(list: &mut [T], id: u32) -> Option<&mut T> {
    if id == 0 {
        None
    } else {
        list.get_mut(id as usize)
    }
}

fn remove_at<T>(list: Option<&mut Vec<T>>, index: usize) -> bool {
    let Some(list) = list else {
        return false;
    };
    if index < list.len() {
        list.remove(index);
        true
    } else {
        false
    }
}

fn slot_step(slot: Slot) -> Step {
    match slot {
        Slot::Then => Step::Then,
        Slot::Else => Step::Else,
        Slot::Choice(index) => Step::Choice(index),
        Slot::Cancel => Step::Cancel,
        Slot::Win => Step::Win,
        Slot::Escape => Step::Escape,
    }
}

fn leading_id(value: &str) -> Option<u32> {
    value.split_whitespace().next()?.parse().ok()
}

fn by_label<T: Copy + std::fmt::Display>(options: &[T], label: &str) -> Option<T> {
    options
        .iter()
        .copied()
        .find(|item| item.to_string() == label)
}

fn labels<T: std::fmt::Display>(items: &[T]) -> Vec<String> {
    items.iter().map(|item| item.to_string()).collect()
}

fn named<T>(list: &[T], id: impl Fn(&T) -> u32, name: impl Fn(&T) -> String) -> Vec<(u32, String)> {
    list.iter()
        .filter(|item| id(item) != 0)
        .map(|item| (id(item), name(item)))
        .collect()
}

fn id_options(pairs: impl IntoIterator<Item = (u32, String)>) -> Vec<String> {
    let mut options = vec!["0 None".into()];
    for (id, name) in pairs {
        if id != 0 {
            options.push(format!("{id} {name}"));
        }
    }
    options
}

fn id_current(id: u32, name: &str) -> String {
    if id == 0 {
        "0 None".into()
    } else {
        format!("{id} {name}")
    }
}

fn heading(label: &str) -> Element<'static, Message> {
    text(label.to_string()).size(22).color(chrome::GOLD).into()
}

fn missing() -> Element<'static, Message> {
    text("Select an entry, or add one.").into()
}

fn btn(label: &str, message: Message) -> Element<'static, Message> {
    button(text(label.to_string())).on_press(message).into()
}

fn field_text(key: &str, label: &str, value: &str) -> Element<'static, Message> {
    let key = key.to_string();
    row![
        text(label.to_string())
            .size(14)
            .color(chrome::MUTED)
            .width(Length::Fixed(140.0)),
        text_input("", value)
            .on_input(move |value| Message::Text {
                key: key.clone(),
                value
            })
            .width(Length::Fill),
    ]
    .spacing(10)
    .align_y(iced::Alignment::Center)
    .into()
}

fn field_num<'a>(
    maker: &'a Maker,
    key: &str,
    label: &str,
    value: impl std::fmt::Display,
) -> Element<'a, Message> {
    let shown = maker.shown(key, value);
    let key = key.to_string();
    row![
        text(label.to_string())
            .size(14)
            .color(chrome::MUTED)
            .width(Length::Fixed(110.0)),
        text_input("0", &shown)
            .on_input(move |value| Message::Num {
                key: key.clone(),
                value
            })
            .width(Length::Fixed(90.0)),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center)
    .into()
}

fn field_pick(
    key: &str,
    label: &str,
    options: Vec<String>,
    current: String,
) -> Element<'static, Message> {
    let selected = options
        .iter()
        .any(|item| item == &current)
        .then_some(current);
    let key = key.to_string();
    row![
        text(label.to_string())
            .size(14)
            .color(chrome::MUTED)
            .width(Length::Fixed(140.0)),
        pick_list(options, selected, move |value| Message::Pick {
            key: key.clone(),
            value
        })
        .width(Length::Fill),
    ]
    .spacing(10)
    .align_y(iced::Alignment::Center)
    .into()
}

fn check(key: &str, label: &str, on: bool) -> Element<'static, Message> {
    let key = key.to_string();
    checkbox(label.to_string(), on)
        .on_toggle(move |on| Message::Toggle {
            key: key.clone(),
            on,
        })
        .into()
}

fn push_params<'a>(
    mut body: Column<'a, Message>,
    maker: &'a Maker,
    prefix: &str,
    params: &[i32],
) -> Column<'a, Message> {
    for index in 0..PARAM_COUNT {
        let value = params.get(index).copied().unwrap_or(0);
        body = body.push(field_num(
            maker,
            &format!("{prefix}.{index}"),
            PARAM_LABELS[index],
            value,
        ));
    }
    body
}

fn push_effects<'a>(
    mut body: Column<'a, Message>,
    maker: &'a Maker,
    owner: &str,
    effects: &[Effect],
) -> Column<'a, Message> {
    body = body.push(text("Effects").size(18));
    for (index, effect) in effects.iter().enumerate() {
        body = body.push(field_pick(
            &format!("effect.kind.{owner}.{index}"),
            "Effect",
            effect_kinds(),
            effect_label(effect),
        ));
        body = body.push(effect_fields(maker, owner, index, effect));
        body = body.push(btn(
            "Remove effect",
            Message::ListDel {
                key: format!("{owner}.effect"),
                index,
            },
        ));
    }
    body = body.push(btn(
        "Add effect",
        Message::ListAdd {
            key: format!("{owner}.effect"),
        },
    ));
    body
}

fn effect_fields<'a>(
    maker: &'a Maker,
    owner: &str,
    index: usize,
    effect: &Effect,
) -> Element<'a, Message> {
    match effect {
        Effect::RecoverHp { percent, flat } | Effect::RecoverMp { percent, flat } => row![
            field_num(
                maker,
                &format!("effect.{owner}.percent.{index}"),
                "Percent",
                *percent
            ),
            field_num(
                maker,
                &format!("effect.{owner}.flat.{index}"),
                "Flat",
                *flat
            ),
        ]
        .spacing(8)
        .into(),
        Effect::AddState { state_id, chance } => row![
            field_num(
                maker,
                &format!("effect.{owner}.state.{index}"),
                "State",
                *state_id
            ),
            field_num(
                maker,
                &format!("effect.{owner}.chance.{index}"),
                "Chance",
                *chance
            ),
        ]
        .spacing(8)
        .into(),
        Effect::RemoveState { state_id } => field_num(
            maker,
            &format!("effect.{owner}.state.{index}"),
            "State",
            *state_id,
        ),
    }
}

fn push_stacks<'a>(
    mut body: Column<'a, Message>,
    maker: &'a Maker,
    kind: &str,
    stacks: &[Stack],
) -> Column<'a, Message> {
    for (index, stack) in stacks.iter().enumerate() {
        body = body.push(
            row![
                field_num(maker, &format!("stack.{kind}.id.{index}"), "Id", stack.id),
                field_num(
                    maker,
                    &format!("stack.{kind}.count.{index}"),
                    "Count",
                    stack.count
                ),
                button(text("Remove")).on_press(Message::Stack {
                    kind: kind.into(),
                    op: "del".into(),
                    index
                }),
            ]
            .spacing(8),
        );
    }
    body = body.push(button(text("Add")).on_press(Message::Stack {
        kind: kind.into(),
        op: "add".into(),
        index: 0,
    }));
    body
}

fn gear_header<'a>(
    title: &str,
    id: u32,
    name: &str,
    description: &str,
    note: &str,
    prefix: &str,
) -> Column<'a, Message> {
    let mut body = Column::new().spacing(8);
    body = body.push(heading(&format!("{title} {id}")));
    body = body.push(field_text(&format!("{prefix}.name"), "Name", name));
    body = body.push(field_text(
        &format!("{prefix}.description"),
        "Description",
        description,
    ));
    body = body.push(field_text(&format!("{prefix}.note"), "Note", note));
    body
}

fn action_fields<'a>(
    maker: &'a Maker,
    index: usize,
    condition: &ActionCondition,
) -> Element<'a, Message> {
    match condition {
        ActionCondition::HpBelow { percent } => {
            field_num(maker, &format!("action.hp.{index}"), "HP below %", *percent)
        }
        ActionCondition::Turn { start, every } => row![
            field_num(maker, &format!("action.turn.{index}"), "Start turn", *start),
            field_num(maker, &format!("action.every.{index}"), "Every", *every),
        ]
        .spacing(8)
        .into(),
        ActionCondition::Always => text("Always available.").into(),
    }
}

fn action_labels() -> Vec<String> {
    vec!["Always".into(), "HP below".into(), "Turn".into()]
}

fn action_current(condition: &ActionCondition) -> String {
    match condition {
        ActionCondition::Always => "Always".into(),
        ActionCondition::HpBelow { .. } => "HP below".into(),
        ActionCondition::Turn { .. } => "Turn".into(),
    }
}

fn effect_kinds() -> Vec<String> {
    vec![
        "Recover HP".into(),
        "Recover MP".into(),
        "Add state".into(),
        "Remove state".into(),
    ]
}

fn effect_label(effect: &Effect) -> String {
    match effect {
        Effect::RecoverHp { .. } => "Recover HP".into(),
        Effect::RecoverMp { .. } => "Recover MP".into(),
        Effect::AddState { .. } => "Add state".into(),
        Effect::RemoveState { .. } => "Remove state".into(),
    }
}

fn effect_from_kind(value: &str) -> Effect {
    match value {
        "Recover MP" => Effect::RecoverMp {
            percent: 0,
            flat: 5,
        },
        "Add state" => Effect::AddState {
            state_id: 1,
            chance: 100,
        },
        "Remove state" => Effect::RemoveState { state_id: 1 },
        _ => Effect::RecoverHp {
            percent: 0,
            flat: 10,
        },
    }
}

fn removal_label(removal: &Removal) -> String {
    match removal {
        Removal::None => "None".into(),
        Removal::Turns { .. } => "After turns".into(),
        Removal::OnDamage => "On damage".into(),
    }
}

fn self_options() -> Vec<String> {
    let mut options = vec!["None".into()];
    options.extend(labels(&SelfSwitch::all()));
    options
}

fn self_current(value: Option<SelfSwitch>) -> String {
    value
        .map(|key| key.to_string())
        .unwrap_or_else(|| "None".into())
}

fn cond_kinds() -> Vec<String> {
    vec![
        "Switch".into(),
        "Variable".into(),
        "Self switch".into(),
        "Item".into(),
        "Gold".into(),
        "Actor in party".into(),
        "Actor HP".into(),
    ]
}

fn cond_kind(cond: &Condition) -> String {
    match cond {
        Condition::Switch { .. } => "Switch".into(),
        Condition::Variable { .. } => "Variable".into(),
        Condition::SelfSwitch { .. } => "Self switch".into(),
        Condition::Item { .. } => "Item".into(),
        Condition::Gold { .. } => "Gold".into(),
        Condition::ActorInParty { .. } => "Actor in party".into(),
        Condition::ActorHp { .. } => "Actor HP".into(),
    }
}

fn operand_kinds() -> Vec<String> {
    vec![
        "Constant".into(),
        "Variable".into(),
        "Random".into(),
        "Gold".into(),
        "Item count".into(),
        "Actor HP".into(),
        "Actor MP".into(),
        "Actor level".into(),
    ]
}

fn operand_kind(operand: &Operand) -> String {
    match operand {
        Operand::Constant { .. } => "Constant".into(),
        Operand::Variable { .. } => "Variable".into(),
        Operand::Random { .. } => "Random".into(),
        Operand::Gold => "Gold".into(),
        Operand::ItemCount { .. } => "Item count".into(),
        Operand::ActorHp { .. } => "Actor HP".into(),
        Operand::ActorMp { .. } => "Actor MP".into(),
        Operand::ActorLevel { .. } => "Actor level".into(),
    }
}

fn dir_options() -> Vec<String> {
    let mut options = vec!["Unchanged".into()];
    options.extend(labels(&Direction::all()));
    options
}

fn dir_current(direction: Option<Direction>) -> String {
    direction
        .map(|dir| dir.to_string())
        .unwrap_or_else(|| "Unchanged".into())
}

fn actor_current(maker: &Maker, target: &ActorTarget) -> String {
    match target {
        ActorTarget::EntireParty => "Entire Party".into(),
        ActorTarget::Actor(id) => {
            let name = maker
                .game
                .actor(*id)
                .map(|actor| actor.name.as_str())
                .unwrap_or("");
            format!("{id} {name}")
        }
    }
}

fn actor_from_label(value: &str) -> ActorTarget {
    if value == "Entire Party" {
        ActorTarget::EntireParty
    } else {
        ActorTarget::Actor(leading_id(value).unwrap_or(1))
    }
}

fn move_current(maker: &Maker, target: &MoveTarget) -> String {
    match target {
        MoveTarget::Player => "Player".into(),
        MoveTarget::ThisEvent => "This Event".into(),
        MoveTarget::Event { id } => {
            let name = maker
                .map()
                .and_then(|map| map.events.iter().find(|event| event.id == *id))
                .map(|event| event.name.as_str())
                .unwrap_or("");
            format!("Event {id} {name}")
        }
    }
}

fn cancel_options(choices: &[String]) -> Vec<String> {
    let mut options = vec!["Disallow".into(), "Branch".into()];
    for (index, choice) in choices.iter().enumerate() {
        options.push(format!("Choice {index}: {choice}"));
    }
    options
}

fn cancel_current(cancel: &CancelMode, choices: &[String]) -> String {
    match cancel {
        CancelMode::Disallow => "Disallow".into(),
        CancelMode::Branch => "Branch".into(),
        CancelMode::Choice(index) => {
            let label = choices.get(*index).map(String::as_str).unwrap_or("");
            format!("Choice {index}: {label}")
        }
    }
}
