//! Map session: movement, events, messages, and the frame update.

use std::collections::BTreeMap;

use fablespace_data::{
    tile_blocked, Direction, Event, Game, Map, MoveRoute, MoveStep, MoveTarget, MoveType, Paths,
    Priority, SelfSwitch, SwitchValue, Trigger,
};

use crate::interpret::{self, TILE_PX};
use crate::party::{self, Bag, Member};
use crate::rng::Rng;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Input {
    pub dir: Option<Direction>,
    pub confirm: bool,
    pub cancel: bool,
    pub menu: bool,
    pub dash: bool,
    /// Left click in 544×416 game pixels.
    pub click: Option<(i32, i32)>,
}

impl Input {
    pub fn confirm() -> Self {
        Self {
            confirm: true,
            ..Self::default()
        }
    }

    pub fn cancel() -> Self {
        Self {
            cancel: true,
            ..Self::default()
        }
    }

    pub fn walk(dir: Direction) -> Self {
        Self {
            dir: Some(dir),
            ..Self::default()
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Title,
    Map,
    GameOver,
}

#[derive(Clone, Debug)]
pub struct ActiveRoute {
    pub steps: Vec<MoveStep>,
    pub index: usize,
    pub repeat: bool,
    pub skippable: bool,
    pub pause: u32,
    pub command: bool,
    pub saved_speed: u8,
    pub saved_through: bool,
}

#[derive(Clone, Debug)]
pub struct Mover {
    pub tile_x: i32,
    pub tile_y: i32,
    pub dir: Direction,
    pub moving: Option<Direction>,
    pub offset: i32,
    pub speed: u8,
    pub through: bool,
    pub direction_fix: bool,
    pub walk_anime: bool,
    pub step_anime: bool,
    pub anim: u16,
    pub freq_wait: u32,
    pub pattern: u8,
    pub character: String,
    pub character_index: u32,
    pub route: Option<ActiveRoute>,
}

impl Default for Mover {
    fn default() -> Self {
        Self {
            tile_x: 0,
            tile_y: 0,
            dir: Direction::Down,
            moving: None,
            offset: 0,
            speed: 2,
            through: false,
            direction_fix: false,
            walk_anime: true,
            step_anime: false,
            anim: 0,
            freq_wait: 0,
            pattern: 1,
            character: String::new(),
            character_index: 0,
            route: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct EvRt {
    pub id: u32,
    pub page: Option<usize>,
    pub seen_page: Option<usize>,
    pub trigger: Trigger,
    pub priority: Priority,
    pub locked: bool,
    pub move_type: MoveType,
    pub frequency: u8,
    pub body: Mover,
}

#[derive(Clone, Debug)]
pub struct Picture {
    pub id: u32,
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub from_x: i32,
    pub from_y: i32,
    pub to_x: i32,
    pub to_y: i32,
    pub frames_left: u32,
    pub frames_total: u32,
}

#[derive(Clone, Debug)]
pub struct MessageUi {
    pub speaker: String,
    pub face: String,
    pub pages: Vec<Vec<String>>,
    pub page: usize,
}

#[derive(Clone, Debug)]
pub struct ChoiceUi {
    pub options: Vec<String>,
    pub cursor: usize,
    pub branches: Vec<Vec<fablespace_data::Command>>,
    pub cancel: fablespace_data::CancelMode,
    pub cancel_branch: Vec<fablespace_data::Command>,
}

#[derive(Clone, Debug)]
pub struct Frame {
    pub commands: Vec<fablespace_data::Command>,
    pub ip: usize,
}

#[derive(Clone, Debug)]
pub enum Wait {
    None,
    Frames(u32),
    Message,
    Choice,
    Route(MoveTarget),
    Menu,
    Save,
    Battle {
        on_win: Vec<fablespace_data::Command>,
        on_escape: Vec<fablespace_data::Command>,
    },
}

#[derive(Clone, Debug)]
pub struct Interp {
    pub stack: Vec<Frame>,
    pub wait: Wait,
    pub map_id: u32,
    pub event_id: Option<u32>,
}

#[derive(Clone, Debug)]
pub struct Process {
    pub map_id: u32,
    pub event_id: Option<u32>,
    pub common_id: Option<u32>,
    pub page: Option<usize>,
    pub interp: Interp,
}

pub struct Session {
    pub game: Game,
    pub maps: Vec<Map>,
    pub paths: Paths,
    pub map_id: u32,
    pub screen: Screen,
    pub party: Vec<Member>,
    pub gold: i32,
    pub bag: Bag,
    pub switches: Vec<bool>,
    pub variables: Vec<i32>,
    pub self_switches: BTreeMap<(u32, u32, u8), bool>,
    pub player: Mover,
    pub events: Vec<EvRt>,
    pub foreground: Option<Process>,
    pub parallels: Vec<Process>,
    pub pictures: Vec<Picture>,
    pub message: Option<MessageUi>,
    pub choice: Option<ChoiceUi>,
    pub battle: Option<crate::battle::Battle>,
    pub menu: Option<crate::menu::Menu>,
    pub save_ui: Option<crate::menu::SaveUi>,
    pub fade: u8,
    pub play_frames: u64,
    pub rng: Rng,
    pub notice: String,
    pub notice_timer: u16,
    pub title_cursor: usize,
    pub quit: bool,
}

impl Session {
    pub fn new(game: Game, maps: Vec<Map>, paths: Paths) -> Self {
        Self {
            game,
            maps,
            paths,
            map_id: 0,
            screen: Screen::Title,
            party: Vec::new(),
            gold: 0,
            bag: Bag::default(),
            switches: Vec::new(),
            variables: Vec::new(),
            self_switches: BTreeMap::new(),
            player: Mover::default(),
            events: Vec::new(),
            foreground: None,
            parallels: Vec::new(),
            pictures: Vec::new(),
            message: None,
            choice: None,
            battle: None,
            menu: None,
            save_ui: None,
            fade: 0,
            play_frames: 0,
            rng: Rng::new(0x5A17_E2),
            notice: String::new(),
            notice_timer: 0,
            title_cursor: 0,
            quit: false,
        }
    }

    pub fn boot(paths: Paths) -> Result<Self, String> {
        let game = fablespace_data::project::load_game(&paths)?;
        let maps = fablespace_data::project::load_maps(&paths)?;
        if maps.is_empty() {
            return Err("the project has no maps".into());
        }
        Ok(Self::new(game, maps, paths))
    }

    pub fn update(&mut self, input: &Input) {
        let input = self.retarget(input);
        let input = &input;
        self.play_frames = self.play_frames.saturating_add(1);
        if self.notice_timer > 0 {
            self.notice_timer -= 1;
            if self.notice_timer == 0 {
                self.notice.clear();
            }
        }
        if self.fade > 0 {
            self.fade -= 1;
        }
        self.tick_flashes();
        self.tick_pictures();
        match self.screen {
            Screen::Title => self.update_title(input),
            Screen::GameOver => {
                if input.confirm || input.cancel {
                    self.to_title();
                }
            }
            Screen::Map => self.update_map(input),
        }
    }

    fn retarget(&mut self, input: &Input) -> Input {
        let Some((x, y)) = input.click else {
            return *input;
        };
        let Some(press) = crate::draw::press_at(self, x, y) else {
            return *input;
        };
        if !self.apply_press(press) {
            return *input;
        }
        Input {
            confirm: true,
            click: None,
            dir: None,
            cancel: false,
            menu: false,
            dash: input.dash,
        }
    }

    fn apply_press(&mut self, press: crate::draw::Press) -> bool {
        use crate::battle::Phase;
        use crate::draw::Press;
        use crate::menu::MenuPage;
        match press {
            Press::Confirm => true,
            Press::Title(index) => {
                self.title_cursor = index.min(2);
                true
            }
            Press::Choice(index) => {
                let Some(choice) = &mut self.choice else {
                    return false;
                };
                if index >= choice.options.len() {
                    return false;
                }
                choice.cursor = index;
                true
            }
            Press::Save(index) => {
                let Some(ui) = &mut self.save_ui else {
                    return false;
                };
                if index >= ui.slots.len() {
                    return false;
                }
                if ui.cursor != index {
                    ui.confirm = false;
                }
                ui.cursor = index;
                true
            }
            Press::MenuRoot(index) => {
                let Some(menu) = &mut self.menu else {
                    return false;
                };
                menu.cursor = index.min(5);
                menu.page = MenuPage::Root;
                true
            }
            Press::MenuItem(index) => match &mut self.menu {
                Some(menu) => match &mut menu.page {
                    MenuPage::Item { cursor, .. } => {
                        *cursor = index;
                        true
                    }
                    _ => false,
                },
                None => false,
            },
            Press::MenuSkill(index) => match &mut self.menu {
                Some(menu) => match &mut menu.page {
                    MenuPage::Skill { cursor, .. } => {
                        *cursor = index;
                        true
                    }
                    _ => false,
                },
                None => false,
            },
            Press::MenuEquip(index) => match &mut self.menu {
                Some(menu) => match &mut menu.page {
                    MenuPage::Equip { slot, .. } => {
                        *slot = index;
                        true
                    }
                    _ => false,
                },
                None => false,
            },
            Press::MenuBag(index) => match &mut self.menu {
                Some(menu) => match &mut menu.page {
                    MenuPage::EquipPick { cursor, .. } => {
                        *cursor = index;
                        true
                    }
                    _ => false,
                },
                None => false,
            },
            Press::MenuEnd(yes) => match &mut self.menu {
                Some(menu) => match menu.page {
                    MenuPage::End { .. } => {
                        menu.page = MenuPage::End { yes };
                        true
                    }
                    _ => false,
                },
                None => false,
            },
            Press::BattleCommand(index) => {
                let Some(battle) = &mut self.battle else {
                    return false;
                };
                if matches!(battle.phase, Phase::Command { .. }) {
                    battle.phase = Phase::Command { cursor: index };
                    true
                } else {
                    false
                }
            }
            Press::BattleSkill(index) => {
                let Some(battle) = &mut self.battle else {
                    return false;
                };
                if matches!(battle.phase, Phase::Skill { .. }) {
                    battle.phase = Phase::Skill { cursor: index };
                    true
                } else {
                    false
                }
            }
            Press::BattleItem(index) => {
                let Some(battle) = &mut self.battle else {
                    return false;
                };
                if matches!(battle.phase, Phase::Item { .. }) {
                    battle.phase = Phase::Item { cursor: index };
                    true
                } else {
                    false
                }
            }
            Press::BattleEnemy(index) => {
                let Some(battle) = &mut self.battle else {
                    return false;
                };
                match &battle.phase {
                    Phase::Target { pending, .. } => {
                        let pending = pending.clone();
                        battle.phase = Phase::Target {
                            cursor: index,
                            pending,
                        };
                        true
                    }
                    _ => false,
                }
            }
        }
    }

    fn update_title(&mut self, input: &Input) {
        if self.save_ui.is_some() {
            self.update_save(input);
            return;
        }
        if input.dir == Some(Direction::Up) {
            self.title_cursor = self.title_cursor.saturating_sub(1);
        }
        if input.dir == Some(Direction::Down) {
            self.title_cursor = (self.title_cursor + 1).min(2);
        }
        if input.confirm {
            match self.title_cursor {
                0 => self.new_game(),
                1 => self.open_save(true),
                _ => self.quit = true,
            }
        }
    }

    fn update_map(&mut self, input: &Input) {
        if self.message.is_some() {
            if input.confirm {
                self.advance_message();
            }
            return;
        }
        if self.choice.is_some() {
            self.update_choice(input);
            return;
        }
        if self.battle.is_some() {
            self.update_battle(input);
            return;
        }
        if self.save_ui.is_some() {
            self.update_save(input);
            return;
        }
        if self.menu.is_some() {
            self.update_menu(input);
            return;
        }

        let allow_input = self.foreground.is_none();
        self.tick_player(input, allow_input);
        self.tick_events();
        self.refresh_event_pages();
        self.sync_parallels();
        self.drive_foreground();
        self.drive_parallels();
        if self.modal() {
            return;
        }
        if self.foreground.is_none() {
            self.start_autorun();
            self.drive_foreground();
            if self.modal() {
                return;
            }
        }
        if self.foreground.is_none() && input.confirm {
            self.try_action();
            self.drive_foreground();
        } else if self.foreground.is_none() && input.menu {
            self.open_menu();
        }
    }

    fn modal(&self) -> bool {
        self.message.is_some()
            || self.choice.is_some()
            || self.battle.is_some()
            || self.menu.is_some()
            || self.save_ui.is_some()
    }

    pub fn new_game(&mut self) {
        let system = self.game.system.clone();
        self.party = system
            .party
            .iter()
            .filter_map(|id| party::member_from_actor(&self.game, *id))
            .collect();
        self.gold = system.start_gold.max(0);
        self.bag = Bag::default();
        for stack in &system.start_items {
            self.bag.gain_item(stack.id, stack.count);
        }
        for stack in &system.start_weapons {
            self.bag.gain_weapon(stack.id, stack.count);
        }
        for stack in &system.start_armors {
            self.bag.gain_armor(stack.id, stack.count);
        }
        self.switches = vec![false; self.game.switches.len().max(1)];
        self.variables = vec![0; self.game.variables.len().max(1)];
        self.self_switches.clear();
        self.pictures.clear();
        self.foreground = None;
        self.parallels.clear();
        self.message = None;
        self.choice = None;
        self.battle = None;
        self.menu = None;
        self.save_ui = None;
        self.play_frames = 0;
        self.map_id = system.start_map_id;
        self.player = Mover {
            tile_x: system.start_x as i32,
            tile_y: system.start_y as i32,
            dir: system.start_dir,
            speed: 2,
            ..Mover::default()
        };
        self.refresh_player_sprite();
        self.rebuild_events();
        self.screen = Screen::Map;
        self.fade = 12;
    }

    pub fn to_title(&mut self) {
        self.screen = Screen::Title;
        self.foreground = None;
        self.parallels.clear();
        self.message = None;
        self.choice = None;
        self.battle = None;
        self.menu = None;
        self.save_ui = None;
        self.pictures.clear();
        self.title_cursor = 0;
        self.events.clear();
    }

    pub fn enter_game_over(&mut self) {
        self.screen = Screen::GameOver;
        self.foreground = None;
        self.parallels.clear();
        self.message = None;
        self.choice = None;
        self.battle = None;
        self.menu = None;
        self.save_ui = None;
    }

    pub fn transfer(&mut self, map_id: u32, x: u32, y: u32, dir: Option<Direction>) {
        if self.maps.iter().all(|map| map.id != map_id) {
            return;
        }
        self.map_id = map_id;
        self.player.tile_x = x as i32;
        self.player.tile_y = y as i32;
        self.player.offset = 0;
        self.player.moving = None;
        self.player.route = None;
        if let Some(dir) = dir {
            self.player.dir = dir;
        }
        self.parallels.clear();
        self.rebuild_events();
    }

    pub(crate) fn refresh_player_sprite(&mut self) {
        if let Some(member) = self.party.first() {
            if let Some(actor) = self.game.actor(member.actor_id) {
                self.player.character = actor.character.clone();
                self.player.character_index = actor.character_index;
            }
        }
    }

    fn rebuild_events(&mut self) {
        let Some(map) = self.current_map().cloned() else {
            self.events.clear();
            return;
        };
        self.events = map.events.iter().map(event_runtime).collect();
        self.refresh_event_pages();
    }

    pub fn current_map(&self) -> Option<&Map> {
        self.maps.iter().find(|map| map.id == self.map_id)
    }

    pub fn map_name(&self, id: u32) -> String {
        self.maps
            .iter()
            .find(|map| map.id == id)
            .map(|map| map.name.clone())
            .unwrap_or_else(|| format!("Map {id}"))
    }

    pub(crate) fn show_lines(&mut self, speaker: String, face: String, lines: Vec<String>) {
        let mut kept: Vec<String> = lines
            .into_iter()
            .map(|line| line.trim_end().to_string())
            .collect();
        while kept.last().is_some_and(|line| line.is_empty()) {
            kept.pop();
        }
        if kept.is_empty() {
            kept.push(String::new());
        }
        let pages = kept.chunks(4).map(|chunk| chunk.to_vec()).collect();
        self.message = Some(MessageUi {
            speaker,
            face,
            pages,
            page: 0,
        });
    }

    fn advance_message(&mut self) {
        let done = if let Some(message) = &mut self.message {
            if message.page + 1 < message.pages.len() {
                message.page += 1;
                false
            } else {
                true
            }
        } else {
            true
        };
        if !done {
            return;
        }
        self.message = None;
        self.clear_wait(interpret::WaitKind::Message);
        if self.battle.is_some() {
            self.after_battle_message();
        } else {
            self.drive_foreground();
            self.drive_parallels();
            self.drive_battle_event();
        }
    }

    fn update_choice(&mut self, input: &Input) {
        let Some(choice) = self.choice.as_mut() else {
            return;
        };
        let len = choice.options.len().max(1);
        if input.dir == Some(Direction::Up) {
            choice.cursor = (choice.cursor + len - 1) % len;
        }
        if input.dir == Some(Direction::Down) {
            choice.cursor = (choice.cursor + 1) % len;
        }
        if input.cancel {
            let choice = self.choice.clone().unwrap();
            match choice.cancel {
                fablespace_data::CancelMode::Disallow => return,
                fablespace_data::CancelMode::Choice(index) => {
                    let branch = choice.branches.get(index).cloned().unwrap_or_default();
                    self.choice = None;
                    self.push_choice_branch(branch);
                    self.drive_foreground();
                    self.drive_parallels();
                }
                fablespace_data::CancelMode::Branch => {
                    let branch = choice.cancel_branch.clone();
                    self.choice = None;
                    self.push_choice_branch(branch);
                    self.drive_foreground();
                    self.drive_parallels();
                }
            }
            return;
        }
        if input.confirm {
            let choice = self.choice.clone().unwrap();
            let branch = choice
                .branches
                .get(choice.cursor)
                .cloned()
                .unwrap_or_default();
            self.choice = None;
            self.push_choice_branch(branch);
            self.drive_foreground();
            self.drive_parallels();
            self.drive_battle_event();
        }
    }

    pub(crate) fn switch(&self, id: u32) -> bool {
        self.switches.get(id as usize).copied().unwrap_or(false)
    }

    pub(crate) fn set_switch(&mut self, id: u32, value: SwitchValue) {
        if id == 0 {
            return;
        }
        if self.switches.len() <= id as usize {
            self.switches.resize(id as usize + 1, false);
        }
        let current = self.switches[id as usize];
        self.switches[id as usize] = value.apply(current);
    }

    pub(crate) fn variable(&self, id: u32) -> i32 {
        self.variables.get(id as usize).copied().unwrap_or(0)
    }

    pub(crate) fn set_variable(&mut self, id: u32, value: i32) {
        if id == 0 {
            return;
        }
        if self.variables.len() <= id as usize {
            self.variables.resize(id as usize + 1, 0);
        }
        self.variables[id as usize] = value;
    }

    pub(crate) fn self_switch(&self, map_id: u32, event_id: u32, key: SelfSwitch) -> bool {
        self.self_switches
            .get(&(map_id, event_id, key.index()))
            .copied()
            .unwrap_or(false)
    }

    pub(crate) fn set_self_switch(
        &mut self,
        map_id: u32,
        event_id: u32,
        key: SelfSwitch,
        on: bool,
    ) {
        let slot = (map_id, event_id, key.index());
        if on {
            self.self_switches.insert(slot, true);
        } else {
            self.self_switches.remove(&slot);
        }
    }

    pub(crate) fn actor_member(&self, actor_id: u32) -> Option<&Member> {
        self.party.iter().find(|member| member.actor_id == actor_id)
    }

    pub(crate) fn knockout_state(&self) -> u32 {
        self.game
            .states
            .iter()
            .find(|state| state.restriction == fablespace_data::Restriction::CannotAct)
            .map(|state| state.id)
            .unwrap_or(0)
    }

    pub(crate) fn notify(&mut self, text: impl Into<String>) {
        self.notice = text.into();
        self.notice_timer = 90;
    }

    fn tick_pictures(&mut self) {
        for picture in &mut self.pictures {
            if picture.frames_left == 0 || picture.frames_total == 0 {
                continue;
            }
            picture.frames_left -= 1;
            let done = picture.frames_total - picture.frames_left;
            picture.x = picture.from_x
                + (picture.to_x - picture.from_x) * done as i32 / picture.frames_total as i32;
            picture.y = picture.from_y
                + (picture.to_y - picture.from_y) * done as i32 / picture.frames_total as i32;
        }
    }

    pub(crate) fn show_picture(&mut self, id: u32, name: String, x: i32, y: i32) {
        self.pictures.retain(|picture| picture.id != id);
        self.pictures.push(Picture {
            id,
            name,
            x,
            y,
            from_x: x,
            from_y: y,
            to_x: x,
            to_y: y,
            frames_left: 0,
            frames_total: 0,
        });
    }

    pub(crate) fn move_picture(&mut self, id: u32, x: i32, y: i32, frames: u32) {
        if let Some(picture) = self.pictures.iter_mut().find(|picture| picture.id == id) {
            picture.from_x = picture.x;
            picture.from_y = picture.y;
            picture.to_x = x;
            picture.to_y = y;
            picture.frames_total = frames.max(1);
            picture.frames_left = frames.max(1);
        }
    }

    fn tick_player(&mut self, input: &Input, allow_input: bool) {
        if self.player.route.is_some() {
            self.tick_route(None);
            return;
        }
        if self.player.moving.is_some() {
            self.advance(None, input.dash);
        }
        if self.player.moving.is_none() && allow_input {
            if let Some(dir) = input.dir {
                self.try_step(None, dir, input.dash);
            }
        }
    }

    fn tick_events(&mut self) {
        let count = self.events.len();
        for index in 0..count {
            let command_route = self.events[index]
                .body
                .route
                .as_ref()
                .is_some_and(|route| route.command);
            if self.events[index].locked && !command_route {
                if self.events[index].body.moving.is_some() {
                    self.advance(Some(index), false);
                }
                continue;
            }
            if self.events[index].body.route.is_some() {
                self.tick_route(Some(index));
                continue;
            }
            if self.events[index].body.moving.is_some() {
                self.advance(Some(index), false);
                continue;
            }
            if !self.events[index].locked {
                self.autonomous(index);
            }
        }
    }

    fn autonomous(&mut self, index: usize) {
        if self.events[index].body.freq_wait > 0 {
            self.events[index].body.freq_wait -= 1;
            return;
        }
        let kind = self.events[index].move_type;
        let wait = freq_frames(self.events[index].frequency);
        match kind {
            MoveType::Fixed | MoveType::Custom => {}
            MoveType::Random => {
                let dir = Direction::all()[self.rng.below(4) as usize];
                self.try_step(Some(index), dir, false);
                self.events[index].body.freq_wait = wait;
            }
            MoveType::Approach => {
                let (x, y) = (
                    self.events[index].body.tile_x,
                    self.events[index].body.tile_y,
                );
                let dir = self.dir_toward(x, y, true);
                self.try_step(Some(index), dir, false);
                self.events[index].body.freq_wait = wait;
            }
        }
    }

    fn tick_route(&mut self, index: Option<usize>) {
        if self.mover(index).moving.is_some() {
            self.advance(index, false);
            return;
        }
        for _ in 0..24 {
            let Some(route) = self.mover(index).route.clone() else {
                return;
            };
            if route.pause > 0 {
                if let Some(route) = self.mover_mut(index).route.as_mut() {
                    route.pause -= 1;
                }
                return;
            }
            if route.index >= route.steps.len() {
                if route.repeat && !route.steps.is_empty() {
                    if let Some(route) = self.mover_mut(index).route.as_mut() {
                        route.index = 0;
                    }
                    continue;
                }
                self.end_route(index);
                return;
            }
            let step = route.steps[route.index].clone();
            if self.apply_route_step(index, step, route.skippable) {
                return;
            }
        }
    }

    /// `true` when the step uses the rest of this frame.
    fn apply_route_step(&mut self, index: Option<usize>, step: MoveStep, skippable: bool) -> bool {
        match step {
            MoveStep::Down
            | MoveStep::Left
            | MoveStep::Right
            | MoveStep::Up
            | MoveStep::Forward
            | MoveStep::TowardPlayer
            | MoveStep::AwayFromPlayer => {
                let Some(dir) = self.step_dir(index, &step) else {
                    self.bump_route(index);
                    return false;
                };
                let tile_before = (self.mover(index).tile_x, self.mover(index).tile_y);
                self.try_step(index, dir, false);
                let moved = self.mover(index).moving.is_some()
                    || (self.mover(index).tile_x, self.mover(index).tile_y) != tile_before;
                if moved {
                    self.bump_route(index);
                    return true;
                }
                if skippable {
                    self.bump_route(index);
                    return false;
                }
                true
            }
            MoveStep::TurnDown
            | MoveStep::TurnLeft
            | MoveStep::TurnRight
            | MoveStep::TurnUp
            | MoveStep::TurnTowardPlayer => {
                if let Some(dir) = self.step_dir(index, &step) {
                    if !self.mover(index).direction_fix {
                        self.mover_mut(index).dir = dir;
                    }
                }
                self.bump_route(index);
                false
            }
            MoveStep::Wait { frames } => {
                if let Some(route) = self.mover_mut(index).route.as_mut() {
                    route.pause = frames.max(1);
                    route.index += 1;
                }
                true
            }
            MoveStep::Speed { speed } => {
                self.mover_mut(index).speed = speed.clamp(1, 5);
                self.bump_route(index);
                false
            }
            MoveStep::Through { on } => {
                self.mover_mut(index).through = on;
                self.bump_route(index);
                false
            }
        }
    }

    fn bump_route(&mut self, index: Option<usize>) {
        if let Some(route) = self.mover_mut(index).route.as_mut() {
            route.index += 1;
        }
    }

    fn end_route(&mut self, index: Option<usize>) {
        let mover = self.mover_mut(index);
        if let Some(route) = mover.route.take() {
            if route.command {
                mover.speed = route.saved_speed;
                mover.through = route.saved_through;
            }
        }
    }

    fn step_dir(&self, index: Option<usize>, step: &MoveStep) -> Option<Direction> {
        let mover = self.mover(index);
        let (x, y, facing) = (mover.tile_x, mover.tile_y, mover.dir);
        match step {
            MoveStep::Down | MoveStep::TurnDown => Some(Direction::Down),
            MoveStep::Left | MoveStep::TurnLeft => Some(Direction::Left),
            MoveStep::Right | MoveStep::TurnRight => Some(Direction::Right),
            MoveStep::Up | MoveStep::TurnUp => Some(Direction::Up),
            MoveStep::Forward => Some(facing),
            MoveStep::TowardPlayer | MoveStep::TurnTowardPlayer => {
                Some(self.dir_toward(x, y, true))
            }
            MoveStep::AwayFromPlayer => Some(self.dir_toward(x, y, false)),
            _ => None,
        }
    }

    fn dir_toward(&self, x: i32, y: i32, toward: bool) -> Direction {
        let dx = self.player.tile_x - x;
        let dy = self.player.tile_y - y;
        let dir = if dx.abs() > dy.abs() {
            if dx >= 0 {
                Direction::Right
            } else {
                Direction::Left
            }
        } else if dy >= 0 {
            Direction::Down
        } else {
            Direction::Up
        };
        if toward {
            dir
        } else {
            opposite(dir)
        }
    }

    fn try_step(&mut self, index: Option<usize>, dir: Direction, dash: bool) {
        let (x, y, through, fixed) = {
            let mover = self.mover(index);
            (
                mover.tile_x,
                mover.tile_y,
                mover.through,
                mover.direction_fix,
            )
        };
        if !fixed {
            self.mover_mut(index).dir = dir;
        }
        let (dx, dy) = dir.delta();
        let nx = x + dx;
        let ny = y + dy;
        let ignore = index.map(|i| self.events[i].id);
        if index.is_some()
            && self.player.tile_x == nx
            && self.player.tile_y == ny
            && self.player.moving.is_none()
        {
            if let Some(i) = index {
                let trigger = self.events[i].trigger;
                let id = self.events[i].id;
                if matches!(trigger, Trigger::EventTouch | Trigger::PlayerTouch) {
                    self.start_event(id);
                }
            }
            return;
        }
        if through || self.passable(nx, ny, ignore) {
            {
                let mover = self.mover_mut(index);
                mover.moving = Some(dir);
                mover.offset = 0;
            }
            self.advance(index, dash);
            return;
        }
        if index.is_none() {
            if let Some(id) = self.solid_event_at(nx, ny) {
                if self.event_trigger(id) == Some(Trigger::PlayerTouch) {
                    self.start_event(id);
                }
            }
        }
    }

    fn advance(&mut self, index: Option<usize>, dash: bool) {
        let speed = {
            let mover = self.mover(index);
            if dash && mover.route.is_none() {
                mover.speed.saturating_add(1).min(5)
            } else {
                mover.speed.clamp(1, 5)
            }
        };
        let mover = self.mover_mut(index);
        let Some(dir) = mover.moving else {
            return;
        };
        mover.offset += speed_px(speed);
        mover.anim = mover.anim.wrapping_add(1);
        if mover.offset >= TILE_PX {
            let (dx, dy) = dir.delta();
            mover.tile_x += dx;
            mover.tile_y += dy;
            mover.offset = 0;
            mover.moving = None;
        }
    }

    pub(crate) fn passable(&self, x: i32, y: i32, ignore: Option<u32>) -> bool {
        let Some(map) = self.current_map() else {
            return false;
        };
        let Some(tileset) = self.game.tileset(map.tileset_id) else {
            return false;
        };
        if map.index(x, y).is_none() {
            return false;
        }
        if tile_blocked(tileset, map.lower_at(x, y), map.upper_at(x, y)) {
            return false;
        }
        for event in &self.events {
            if Some(event.id) == ignore {
                continue;
            }
            if event.body.tile_x != x || event.body.tile_y != y {
                continue;
            }
            if event.body.through
                || event.priority != Priority::Same
                || event.body.character.is_empty()
            {
                continue;
            }
            return false;
        }
        true
    }

    fn solid_event_at(&self, x: i32, y: i32) -> Option<u32> {
        self.events
            .iter()
            .find(|event| {
                event.body.tile_x == x
                    && event.body.tile_y == y
                    && !event.body.through
                    && event.priority == Priority::Same
                    && !event.body.character.is_empty()
            })
            .map(|event| event.id)
    }

    fn event_trigger(&self, id: u32) -> Option<Trigger> {
        self.events
            .iter()
            .find(|event| event.id == id)
            .map(|event| event.trigger)
    }

    fn mover(&self, index: Option<usize>) -> &Mover {
        match index {
            None => &self.player,
            Some(index) => &self.events[index].body,
        }
    }

    fn mover_mut(&mut self, index: Option<usize>) -> &mut Mover {
        match index {
            None => &mut self.player,
            Some(index) => &mut self.events[index].body,
        }
    }

    pub(crate) fn install_route(&mut self, target: MoveTarget, route: MoveRoute) {
        let slot = match target {
            MoveTarget::Player => Some(None),
            MoveTarget::Event { id } => self
                .events
                .iter()
                .position(|event| event.id == id)
                .map(Some),
            MoveTarget::ThisEvent => None,
        };
        let Some(slot) = slot else {
            return;
        };
        let mover = self.mover_mut(slot);
        let saved_speed = mover.speed;
        let saved_through = mover.through;
        mover.moving = None;
        mover.offset = 0;
        mover.route = Some(ActiveRoute {
            steps: route.steps,
            index: 0,
            repeat: route.repeat,
            skippable: route.skippable,
            pause: 0,
            command: true,
            saved_speed,
            saved_through,
        });
    }

    pub(crate) fn event_route_command(&self, id: u32) -> bool {
        self.events.iter().any(|event| {
            event.id == id && event.body.route.as_ref().is_some_and(|route| route.command)
        })
    }

    pub(crate) fn refresh_event_pages(&mut self) {
        let Some(map) = self.current_map().cloned() else {
            return;
        };
        let picks: Vec<Option<usize>> = self
            .events
            .iter()
            .map(|event| {
                let source = map.events.iter().find(|source| source.id == event.id)?;
                source
                    .pages
                    .iter()
                    .enumerate()
                    .rev()
                    .find(|(_, page)| {
                        page_matches(
                            &page.conditions,
                            &self_switch_lookup(&self.self_switches, self.map_id, event.id),
                            &self.switches,
                            &self.variables,
                            &self.bag,
                            &self.party,
                        )
                    })
                    .map(|(index, _)| index)
            })
            .collect();
        for (event, pick) in self.events.iter_mut().zip(picks) {
            let Some(source) = map.events.iter().find(|source| source.id == event.id) else {
                continue;
            };
            let Some(index) = pick else {
                event.page = None;
                event.body.character.clear();
                event.priority = Priority::Below;
                continue;
            };
            let Some(page) = source.pages.get(index) else {
                continue;
            };
            event.page = Some(index);
            event.trigger = page.trigger;
            event.priority = page.priority;
            event.body.character = page.image.character.clone();
            event.body.character_index = page.image.index;
            event.body.through = page.through;
            event.body.direction_fix = page.direction_fix;
            event.body.walk_anime = page.walk_anime;
            event.body.step_anime = page.step_anime;
            event.body.speed = page.move_speed.clamp(1, 5);
            event.frequency = page.move_frequency;
            event.move_type = page.move_type;
            if event.locked || event.seen_page == Some(index) {
                continue;
            }
            event.seen_page = Some(index);
            event.body.dir = page.image.direction;
            event.body.pattern = page.image.pattern;
            event.body.moving = None;
            event.body.offset = 0;
            event.body.route = if page.move_type == MoveType::Custom && !page.route.steps.is_empty()
            {
                Some(ActiveRoute {
                    steps: page.route.steps.clone(),
                    index: 0,
                    repeat: true,
                    skippable: true,
                    pause: 0,
                    command: false,
                    saved_speed: page.move_speed,
                    saved_through: page.through,
                })
            } else {
                None
            };
        }
    }

    fn start_event(&mut self, id: u32) {
        if self.foreground.is_some() || self.battle.is_some() {
            return;
        }
        let map_id = self.map_id;
        let Some(map) = self.current_map().cloned() else {
            return;
        };
        let Some(source) = map.events.iter().find(|event| event.id == id) else {
            return;
        };
        let Some(runtime) = self.events.iter_mut().find(|event| event.id == id) else {
            return;
        };
        let Some(page_index) = runtime.page else {
            return;
        };
        let Some(page) = source.pages.get(page_index) else {
            return;
        };
        runtime.locked = true;
        let commands = page.list.clone();
        self.foreground = Some(Process::event(map_id, id, commands));
        if let Some(process) = &mut self.foreground {
            process.page = Some(page_index);
        }
    }

    fn try_action(&mut self) {
        if self.player.moving.is_some() || self.foreground.is_some() {
            return;
        }
        let (dx, dy) = self.player.dir.delta();
        let nx = self.player.tile_x + dx;
        let ny = self.player.tile_y + dy;
        if let Some(id) = self.solid_event_at(nx, ny) {
            let trigger = self.event_trigger(id);
            if matches!(trigger, Some(Trigger::Action | Trigger::PlayerTouch)) {
                self.start_event(id);
            }
        }
    }

    fn start_autorun(&mut self) {
        if self.foreground.is_some() {
            return;
        }
        let map_id = self.map_id;
        if let Some(id) = self
            .events
            .iter()
            .find(|event| event.trigger == Trigger::Autorun && event.page.is_some())
            .map(|event| event.id)
        {
            self.start_event(id);
            return;
        }
        if let Some(event) = self.game.common_events.iter().find(|event| {
            event.trigger == fablespace_data::CommonTrigger::Autorun
                && event.id != 0
                && (event.switch_id == 0 || self.switch(event.switch_id))
        }) {
            let id = event.id;
            let list = event.list.clone();
            self.foreground = Some(Process::common(map_id, id, list));
        }
    }

    fn sync_parallels(&mut self) {
        let map_id = self.map_id;
        let mut wanted: Vec<(u64, Process)> = Vec::new();
        for event in &self.events {
            if event.trigger != Trigger::Parallel || event.page.is_none() {
                continue;
            }
            let Some(source) = self
                .current_map()
                .and_then(|map| map.events.iter().find(|source| source.id == event.id))
            else {
                continue;
            };
            let Some(page) = event.page.and_then(|index| source.pages.get(index)) else {
                continue;
            };
            let key = event.id as u64;
            let mut process = Process::event(map_id, event.id, page.list.clone());
            process.page = event.page;
            wanted.push((key, process));
        }
        for event in self.game.common_events.clone() {
            if event.id == 0 || event.trigger != fablespace_data::CommonTrigger::Parallel {
                continue;
            }
            if event.switch_id != 0 && !self.switch(event.switch_id) {
                continue;
            }
            let mut process = Process::common(map_id, event.id, event.list.clone());
            process.page = Some(0);
            wanted.push((10_000 + event.id as u64, process));
        }
        let mut next = Vec::new();
        for (key, fresh) in wanted {
            if let Some(existing) = self
                .parallels
                .iter()
                .find(|process| parallel_key(process) == key)
            {
                if existing.page == fresh.page {
                    next.push(existing.clone());
                    continue;
                }
            }
            next.push(fresh);
        }
        self.parallels = next;
    }

    pub(crate) fn unlock_event(&mut self, id: u32) {
        if let Some(event) = self.events.iter_mut().find(|event| event.id == id) {
            event.locked = false;
            event.seen_page = None;
        }
        self.refresh_event_pages();
    }

    pub fn sprite_pixel(mover: &Mover) -> (i32, i32) {
        let (dx, dy) = mover.moving.map(Direction::delta).unwrap_or((0, 0));
        (
            mover.tile_x * TILE_PX + dx * mover.offset,
            mover.tile_y * TILE_PX + dy * mover.offset,
        )
    }

    pub fn anim_pattern(mover: &Mover) -> u8 {
        let animate = (mover.moving.is_some() && mover.walk_anime) || mover.step_anime;
        if animate {
            const CYCLE: [u8; 4] = [1, 0, 1, 2];
            CYCLE[(mover.anim as usize / 6) % 4]
        } else {
            mover.pattern
        }
    }
}

fn parallel_key(process: &Process) -> u64 {
    if let Some(id) = process.common_id {
        10_000 + id as u64
    } else {
        process.event_id.unwrap_or(0) as u64
    }
}

fn event_runtime(event: &Event) -> EvRt {
    EvRt {
        id: event.id,
        page: None,
        seen_page: None,
        trigger: Trigger::Action,
        priority: Priority::Same,
        locked: false,
        move_type: MoveType::Fixed,
        frequency: 3,
        body: Mover {
            tile_x: event.x as i32,
            tile_y: event.y as i32,
            ..Mover::default()
        },
    }
}

fn self_switch_lookup(
    map: &BTreeMap<(u32, u32, u8), bool>,
    map_id: u32,
    event_id: u32,
) -> impl Fn(SelfSwitch) -> bool + '_ {
    move |key| {
        map.get(&(map_id, event_id, key.index()))
            .copied()
            .unwrap_or(false)
    }
}

fn page_matches(
    cond: &fablespace_data::PageConditions,
    self_switch: &impl Fn(SelfSwitch) -> bool,
    switches: &[bool],
    variables: &[i32],
    bag: &Bag,
    party: &[Member],
) -> bool {
    if cond.switch_1 != 0
        && !switches
            .get(cond.switch_1 as usize)
            .copied()
            .unwrap_or(false)
    {
        return false;
    }
    if cond.switch_2 != 0
        && !switches
            .get(cond.switch_2 as usize)
            .copied()
            .unwrap_or(false)
    {
        return false;
    }
    if cond.variable_id != 0
        && variables
            .get(cond.variable_id as usize)
            .copied()
            .unwrap_or(0)
            < cond.variable_min
    {
        return false;
    }
    if let Some(key) = cond.self_switch {
        if !self_switch(key) {
            return false;
        }
    }
    if cond.item_id != 0 && bag.count_item(cond.item_id) == 0 {
        return false;
    }
    if cond.actor_id != 0 && !party.iter().any(|member| member.actor_id == cond.actor_id) {
        return false;
    }
    true
}

fn freq_frames(freq: u8) -> u32 {
    match freq {
        1 => 80,
        2 => 52,
        3 => 32,
        4 => 18,
        _ => 10,
    }
}

fn speed_px(speed: u8) -> i32 {
    // A tile is 32px. Speed 2, the walk, takes 16 frames, so releasing the
    // key lands on the next tile instead of several tiles later. Dash is
    // one step faster and still finishes on a tile.
    match speed {
        1 => 1,
        2 => 2,
        _ => 4,
    }
}

fn opposite(dir: Direction) -> Direction {
    match dir {
        Direction::Down => Direction::Up,
        Direction::Up => Direction::Down,
        Direction::Left => Direction::Right,
        Direction::Right => Direction::Left,
    }
}
