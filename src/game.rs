//! Walk a field, talk, and fight.
//!
//! The map is 120x70 tiles drawn at 2x, so a 1920x1080 view shows about
//! 60x33 tiles. The camera follows Mira and stops at the edges. The west
//! and center are grass. A dirt road on the east runs north into two
//! buildings. The game opens on a title screen. Esc is the system menu
//! and Tab is the party menu.
//!
//! Attack damage is `max(1, atk - def)`. Guard halves that damage, to a
//! minimum of 1. The player's attack includes the equipped weapon.
//!
//! Win removes the monster. Flee leaves both of you as you are. A loss
//! restores the player's HP only; the monster stays, at the HP it had,
//! so the fight can be tried again.

use crate::atlas::TILE;
use std::fmt::Write as _;
use std::fs;
use std::io;
use std::path::PathBuf;

pub const WIDTH: i32 = 1920;
pub const HEIGHT: i32 = 1080;
pub const MAP_W: i32 = 120;
pub const MAP_H: i32 = 70;
/// Field tiles and sprites are drawn at this multiple of the 16px art.
/// 2x is 32px per tile, so the 1920x1080 view shows 60 x 33 tiles.
pub const SCALE: i32 = 2;
pub const WALK_FRAMES: u8 = 8;
pub const TRANSITION_FRAMES: u16 = 36;
pub const BEAT: u8 = 28;
pub const LEAVE: u8 = 42;

pub const TALK_LINE: &str = "Keep off my grass.";
pub const COMMANDS: [&str; 3] = ["Attack", "Guard", "Flee"];
pub const WEAPON_NAME: &str = "Short Sword";
pub const WEAPON_ATK: i32 = 3;

pub const START_X: i32 = 40;
pub const START_Y: i32 = 40;
pub const SLIME_X: i32 = 48;
pub const SLIME_Y: i32 = 35;

/// North-south dirt road on the east side. It stops at the south edge of town.
pub const ROAD_X: i32 = 99;
pub const ROAD_W: i32 = 3;
pub const ROAD_Y0: i32 = 14;
pub const ROAD_Y1: i32 = 62;

pub const TITLE_ITEMS: [&str; 4] = ["Continue", "Load Game", "New Game", "Quit"];
pub const SYSTEM_ITEMS: [&str; 6] = [
    "Save",
    "Load",
    "Audio",
    "Resolution",
    "Save and Quit",
    "Quit",
];

const SAVE_ITEM: usize = 0;
const LOAD_ITEM: usize = 1;
const AUDIO_ITEM: usize = 2;
const RESOLUTION_ITEM: usize = 3;
const SAVE_QUIT_ITEM: usize = 4;
const QUIT_ITEM: usize = 5;

pub const RESOLUTIONS: [(u32, u32); 4] = [(1280, 720), (1600, 900), (1920, 1080), (2560, 1440)];

/// Two buildings side by side, north of the road. `wood` picks the tile set.
#[derive(Clone, Copy, Debug)]
pub struct Building {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub roof_rows: i32,
    pub wood: bool,
}

pub const BUILDINGS: [Building; 2] = [
    Building {
        x: 93,
        y: 6,
        w: 7,
        h: 8,
        roof_rows: 3,
        wood: true,
    },
    Building {
        x: 101,
        y: 6,
        w: 7,
        h: 8,
        roof_rows: 3,
        wood: false,
    },
];

pub const MUSHROOMS: [(i32, i32); 4] = [(22, 38), (30, 32), (55, 44), (36, 50)];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Terrain {
    Grass,
    RoadLeft,
    Road,
    RoadRight,
    RoofRed,
    WallWood,
    RoofBlue,
    WallStone,
}

#[derive(Clone, Debug)]
pub struct Item {
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct Character {
    pub name: String,
    pub weapon_name: String,
    pub weapon_atk: i32,
    pub inventory: Vec<Item>,
    pub skills: Vec<String>,
}

#[derive(Clone, Copy, Debug)]
pub struct Fighter {
    pub name: &'static str,
    pub hp: i32,
    pub max_hp: i32,
    pub atk: i32,
    pub def: i32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Input {
    /// Held direction. -1, 0, or 1. Up and left are negative.
    pub held_x: i32,
    pub held_y: i32,
    /// One-shot direction from a key press. Cleared by the caller each frame.
    pub tap_x: i32,
    pub tap_y: i32,
    pub confirm: bool,
    pub pause: bool,
    pub tab: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Back {
    Field,
    Talk,
    Battle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Title,
    LoadList { from_title: bool, back: Back },
    Field,
    Talk,
    Transition { frame: u16 },
    Battle,
    System(Back),
    QuitAsk(Back),
    Party,
    Skills,
    Inventory,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Choose,
    AfterPlayer { timer: u8 },
    AfterEnemy { timer: u8 },
    Leave { timer: u8, outcome: Outcome },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Win,
    Flee,
    Lose,
}

#[derive(Clone, Debug)]
pub struct Battle {
    pub cursor: usize,
    pub log: Vec<String>,
    pub phase: Phase,
    pub guarding: bool,
}

#[derive(Clone, Copy, Debug)]
struct Walk {
    from_x: i32,
    from_y: i32,
    t: u8,
}

struct SaveData {
    px: Option<i32>,
    py: Option<i32>,
    hp: Option<i32>,
    max_hp: Option<i32>,
    slime_alive: Option<bool>,
    slime_hp: Option<i32>,
    slime_max: Option<i32>,
    slime_x: Option<i32>,
    slime_y: Option<i32>,
    volume: Option<u8>,
    width: Option<u32>,
    height: Option<u32>,
    characters: Vec<Character>,
}

pub struct Game {
    pub mode: Mode,
    pub player: Fighter,
    pub monster: Fighter,
    pub monster_alive: bool,
    pub px: i32,
    pub py: i32,
    pub mx: i32,
    pub my: i32,
    pub battle: Battle,
    pub menu_cursor: usize,
    pub party_index: usize,
    pub volume: u8,
    pub window_w: u32,
    pub window_h: u32,
    pub notice: Option<&'static str>,
    pub characters: Vec<Character>,
    pub slots: Vec<u32>,
    walk: Option<Walk>,
    queued: Option<(i32, i32)>,
    quit: bool,
    save_dir: PathBuf,
}

pub fn terrain_at(x: i32, y: i32) -> Option<Terrain> {
    if x < 0 || y < 0 || x >= MAP_W || y >= MAP_H {
        return None;
    }
    for building in BUILDINGS {
        if x >= building.x
            && x < building.x + building.w
            && y >= building.y
            && y < building.y + building.h
        {
            let roof = y < building.y + building.roof_rows;
            return Some(match (building.wood, roof) {
                (true, true) => Terrain::RoofRed,
                (true, false) => Terrain::WallWood,
                (false, true) => Terrain::RoofBlue,
                (false, false) => Terrain::WallStone,
            });
        }
    }
    if x >= ROAD_X && x < ROAD_X + ROAD_W && y >= ROAD_Y0 && y < ROAD_Y1 {
        return Some(match x - ROAD_X {
            0 => Terrain::RoadLeft,
            1 => Terrain::Road,
            _ => Terrain::RoadRight,
        });
    }
    Some(Terrain::Grass)
}

pub fn ground_at(x: i32, y: i32) -> bool {
    matches!(
        terrain_at(x, y),
        Some(Terrain::Grass | Terrain::RoadLeft | Terrain::Road | Terrain::RoadRight)
    )
}

impl Game {
    pub fn new() -> Self {
        Self::with_save_dir(default_save_dir())
    }

    pub fn with_save_dir(dir: impl Into<PathBuf>) -> Self {
        let mut game = Self::fresh(dir.into());
        game.load_settings();
        game
    }

    pub fn start_new_game(&mut self) {
        let volume = self.volume;
        let window_w = self.window_w;
        let window_h = self.window_h;
        let save_dir = self.save_dir.clone();
        *self = Self::fresh(save_dir);
        self.volume = volume;
        self.window_w = window_w;
        self.window_h = window_h;
        self.mode = Mode::Field;
    }

    pub fn update(&mut self, input: &Input) {
        if self.quit {
            return;
        }
        match self.mode {
            Mode::Title => self.update_title(input),
            Mode::LoadList { .. } => self.update_load(input),
            Mode::System(_) => self.update_system(input),
            Mode::QuitAsk(_) => self.update_quit_ask(input),
            Mode::Party => self.update_party(input),
            Mode::Skills | Mode::Inventory => self.update_submenu(input),
            Mode::Field | Mode::Talk | Mode::Battle | Mode::Transition { .. } => {
                self.update_world(input);
            }
        }
    }

    pub fn should_quit(&self) -> bool {
        self.quit
    }

    pub fn window_size(&self) -> (u32, u32) {
        (self.window_w, self.window_h)
    }

    /// Master volume as a gain from 0 to 1. Menu changes update it the same frame.
    pub fn master_gain(&self) -> f32 {
        f32::from(self.volume) / 10.0
    }

    pub fn selected(&self) -> &Character {
        let index = self
            .party_index
            .min(self.characters.len().saturating_sub(1));
        &self.characters[index]
    }

    pub fn near_monster(&self) -> bool {
        self.monster_alive
            && self.walk.is_none()
            && (self.px - self.mx).abs() + (self.py - self.my).abs() == 1
    }

    pub fn player_px(&self) -> (i32, i32) {
        match self.walk {
            Some(walk) => {
                let t = i32::from(walk.t);
                let n = i32::from(WALK_FRAMES);
                let x = walk.from_x * TILE + (self.px - walk.from_x) * TILE * t / n;
                let y = walk.from_y * TILE + (self.py - walk.from_y) * TILE * t / n;
                (x, y)
            }
            None => (self.px * TILE, self.py * TILE),
        }
    }

    pub fn bobbing(&self) -> bool {
        self.walk.is_some_and(|walk| walk.t % 8 < 4)
    }

    /// Top-left of the view, in screen pixels. The camera keeps the player
    /// in frame and stops at the edge of the map.
    pub fn camera(&self) -> (i32, i32) {
        let (x, y) = self.player_px();
        let tile = TILE * SCALE;
        let focus_x = x * SCALE + tile / 2;
        let focus_y = y * SCALE + tile / 2;
        (
            scroll_axis(focus_x, MAP_W * tile, WIDTH),
            scroll_axis(focus_y, MAP_H * tile, HEIGHT),
        )
    }

    /// Ground the player can stand on. A building, or the space past the
    /// edge of the map, is not ground and draws as a flat color when no
    /// tile covers it.
    pub fn ground(&self, x: i32, y: i32) -> bool {
        ground_at(x, y)
    }

    fn fresh(save_dir: PathBuf) -> Self {
        Self {
            mode: Mode::Title,
            player: Fighter {
                name: "Mira",
                hp: 20,
                max_hp: 20,
                atk: 4,
                def: 2,
            },
            monster: Fighter {
                name: "Slime",
                hp: 12,
                max_hp: 12,
                atk: 5,
                def: 1,
            },
            monster_alive: true,
            px: START_X,
            py: START_Y,
            mx: SLIME_X,
            my: SLIME_Y,
            battle: Battle::fresh("Slime"),
            menu_cursor: 0,
            party_index: 0,
            volume: 10,
            window_w: WIDTH as u32,
            window_h: HEIGHT as u32,
            notice: None,
            characters: vec![mira()],
            slots: Vec::new(),
            walk: None,
            queued: None,
            quit: false,
            save_dir,
        }
    }

    fn update_title(&mut self, input: &Input) {
        if input.tap_y != 0 {
            self.notice = None;
            self.step_cursor(input.tap_y, TITLE_ITEMS.len());
        }
        if !input.confirm {
            return;
        }
        match self.menu_cursor {
            0 => {
                if !self.load_newest() {
                    self.notice = Some("No save.");
                }
            }
            1 => self.open_load(true, Back::Field),
            2 => self.start_new_game(),
            _ => self.quit = true,
        }
    }

    fn update_load(&mut self, input: &Input) {
        let (from_title, back) = match self.mode {
            Mode::LoadList { from_title, back } => (from_title, back),
            _ => return,
        };
        if input.pause {
            self.close_load(from_title, back);
            return;
        }
        if self.slots.is_empty() {
            return;
        }
        if input.tap_y != 0 {
            self.notice = None;
            self.step_cursor(input.tap_y, self.slots.len());
        }
        if input.confirm {
            let index = self.menu_cursor.min(self.slots.len() - 1);
            let id = self.slots[index];
            if !self.load_id(id) {
                self.notice = Some("Save failed.");
            }
        }
    }

    fn close_load(&mut self, from_title: bool, back: Back) {
        self.notice = None;
        if from_title {
            self.menu_cursor = 1;
            self.mode = Mode::Title;
        } else {
            self.menu_cursor = LOAD_ITEM;
            self.mode = Mode::System(back);
        }
    }

    fn update_system(&mut self, input: &Input) {
        let back = match self.mode {
            Mode::System(back) => back,
            _ => return,
        };
        if input.pause {
            self.notice = None;
            self.mode = mode_of(back);
            return;
        }
        if input.tap_y != 0 {
            self.notice = None;
            self.step_cursor(input.tap_y, SYSTEM_ITEMS.len());
        }
        match self.menu_cursor {
            AUDIO_ITEM if input.tap_x != 0 => self.step_volume(input.tap_x),
            RESOLUTION_ITEM if input.tap_x != 0 => self.step_resolution(input.tap_x),
            _ => {}
        }
        if !input.confirm {
            return;
        }
        match self.menu_cursor {
            SAVE_ITEM => {
                self.notice = if self.write_save().is_ok() {
                    Some("Saved.")
                } else {
                    Some("Save failed.")
                };
            }
            LOAD_ITEM => self.open_load(false, back),
            SAVE_QUIT_ITEM => {
                if self.write_save().is_ok() {
                    self.quit = true;
                } else {
                    self.notice = Some("Save failed.");
                }
            }
            QUIT_ITEM => {
                self.menu_cursor = 0;
                self.notice = None;
                self.mode = Mode::QuitAsk(back);
            }
            _ => {}
        }
    }

    fn update_quit_ask(&mut self, input: &Input) {
        let back = match self.mode {
            Mode::QuitAsk(back) => back,
            _ => return,
        };
        if input.pause {
            self.menu_cursor = QUIT_ITEM;
            self.notice = None;
            self.mode = Mode::System(back);
            return;
        }
        if input.tap_y != 0 {
            self.step_cursor(input.tap_y, 2);
        }
        if !input.confirm {
            return;
        }
        if self.menu_cursor == 0 {
            if self.write_save().is_ok() {
                self.quit = true;
            } else {
                self.notice = Some("Save failed.");
            }
        } else {
            self.quit = true;
        }
    }

    fn update_party(&mut self, input: &Input) {
        if input.pause || input.tab {
            self.mode = Mode::Field;
            return;
        }
        if self.menu_cursor == 0 && input.tap_x != 0 && !self.characters.is_empty() {
            let count = self.characters.len() as i32;
            let next = self.party_index as i32 + input.tap_x.signum();
            self.party_index = next.rem_euclid(count) as usize;
        }
        if input.tap_y != 0 {
            self.step_cursor(input.tap_y, 3);
        }
        if input.confirm {
            self.mode = match self.menu_cursor {
                1 => Mode::Skills,
                2 => Mode::Inventory,
                _ => Mode::Party,
            };
        }
    }

    fn update_submenu(&mut self, input: &Input) {
        if input.tab {
            self.mode = Mode::Field;
        } else if input.pause {
            self.mode = Mode::Party;
        }
    }

    fn update_world(&mut self, input: &Input) {
        let in_transition = matches!(self.mode, Mode::Transition { .. });
        if !in_transition && input.pause {
            let back = match self.mode {
                Mode::Talk => Back::Talk,
                Mode::Battle => Back::Battle,
                _ => Back::Field,
            };
            self.menu_cursor = 0;
            self.notice = None;
            self.mode = Mode::System(back);
            return;
        }
        if input.tab && self.mode == Mode::Field {
            self.menu_cursor = 0;
            self.mode = Mode::Party;
            return;
        }
        match self.mode {
            Mode::Field => {
                if input.confirm && self.near_monster() {
                    self.queued = None;
                    self.mode = Mode::Talk;
                } else {
                    self.move_player(input);
                }
            }
            Mode::Talk => {
                if input.confirm {
                    self.begin_fight();
                }
            }
            Mode::Transition { frame } => {
                let next = frame + 1;
                if next >= TRANSITION_FRAMES {
                    self.mode = Mode::Battle;
                } else {
                    self.mode = Mode::Transition { frame: next };
                }
            }
            Mode::Battle => self.update_battle(input),
            _ => {}
        }
    }

    fn step_cursor(&mut self, tap_y: i32, len: usize) {
        if len == 0 {
            return;
        }
        if tap_y < 0 {
            self.menu_cursor = (self.menu_cursor + len - 1) % len;
        } else if tap_y > 0 {
            self.menu_cursor = (self.menu_cursor + 1) % len;
        }
    }

    fn step_volume(&mut self, dir: i32) {
        let next = i32::from(self.volume) + dir.signum();
        self.volume = next.clamp(0, 10) as u8;
        let _ = self.write_settings();
    }

    fn step_resolution(&mut self, dir: i32) {
        let count = RESOLUTIONS.len() as i32;
        let index = (self.resolution_index() as i32 + dir.signum()).rem_euclid(count) as usize;
        let (width, height) = RESOLUTIONS[index];
        self.window_w = width;
        self.window_h = height;
        let _ = self.write_settings();
    }

    fn resolution_index(&self) -> usize {
        RESOLUTIONS
            .iter()
            .position(|size| *size == (self.window_w, self.window_h))
            .unwrap_or(2)
    }

    fn open_load(&mut self, from_title: bool, back: Back) {
        self.slots = self.list_saves();
        self.menu_cursor = 0;
        self.notice = None;
        self.mode = Mode::LoadList { from_title, back };
    }

    fn equipped_atk(&self) -> i32 {
        self.selected().weapon_atk
    }

    fn begin_fight(&mut self) {
        self.battle = Battle::fresh(self.monster.name);
        self.queued = None;
        self.mode = Mode::Transition { frame: 0 };
    }

    fn move_player(&mut self, input: &Input) {
        self.tick_walk();
        if self.walk.is_some() {
            if input.tap_x != 0 || input.tap_y != 0 {
                self.queued = Some((input.tap_x.signum(), input.tap_y.signum()));
            }
            return;
        }
        let (dx, dy) = if let Some(step) = self.queued.take() {
            step
        } else if input.tap_x != 0 || input.tap_y != 0 {
            (input.tap_x.signum(), input.tap_y.signum())
        } else {
            (input.held_x.signum(), input.held_y.signum())
        };
        if dx != 0 && dy != 0 {
            if self.try_step(dx, dy) {
                return;
            }
            if self.try_step(dx, 0) {
                return;
            }
            let _ = self.try_step(0, dy);
        } else if dx != 0 || dy != 0 {
            let _ = self.try_step(dx, dy);
        }
    }

    fn tick_walk(&mut self) {
        let done = if let Some(walk) = self.walk.as_mut() {
            walk.t = walk.t.saturating_add(1);
            walk.t >= WALK_FRAMES
        } else {
            false
        };
        if done {
            self.walk = None;
        }
    }

    fn try_step(&mut self, dx: i32, dy: i32) -> bool {
        if self.walk.is_some() || (dx == 0 && dy == 0) {
            return false;
        }
        let nx = self.px + dx;
        let ny = self.py + dy;
        if !self.tile_free(nx, ny) {
            return false;
        }
        self.walk = Some(Walk {
            from_x: self.px,
            from_y: self.py,
            t: 0,
        });
        self.px = nx;
        self.py = ny;
        true
    }

    fn tile_free(&self, x: i32, y: i32) -> bool {
        self.ground(x, y) && !(self.monster_alive && x == self.mx && y == self.my)
    }

    fn update_battle(&mut self, input: &Input) {
        match self.battle.phase {
            Phase::Choose => {
                if input.tap_y < 0 {
                    self.battle.cursor = (self.battle.cursor + COMMANDS.len() - 1) % COMMANDS.len();
                } else if input.tap_y > 0 {
                    self.battle.cursor = (self.battle.cursor + 1) % COMMANDS.len();
                }
                if input.confirm {
                    self.player_command();
                }
            }
            Phase::AfterPlayer { timer } => {
                let next = timer.saturating_sub(1);
                if next == 0 {
                    let phase = self.enemy_turn();
                    self.battle.phase = phase;
                } else {
                    self.battle.phase = Phase::AfterPlayer { timer: next };
                }
            }
            Phase::AfterEnemy { timer } => {
                let next = timer.saturating_sub(1);
                self.battle.phase = if next == 0 {
                    Phase::Choose
                } else {
                    Phase::AfterEnemy { timer: next }
                };
            }
            Phase::Leave { timer, outcome } => {
                let next = timer.saturating_sub(1);
                if next == 0 {
                    self.apply_outcome(outcome);
                } else {
                    self.battle.phase = Phase::Leave {
                        timer: next,
                        outcome,
                    };
                }
            }
        }
    }

    fn player_command(&mut self) {
        match self.battle.cursor {
            0 => {
                let dmg = harm(
                    self.player.atk + self.equipped_atk(),
                    self.monster.def,
                    false,
                );
                self.monster.hp = (self.monster.hp - dmg).max(0);
                self.push_log(format!("{} hits for {dmg}.", self.player.name));
                if self.monster.hp == 0 {
                    self.push_log(format!("{} melts.", self.monster.name));
                    self.battle.phase = Phase::Leave {
                        timer: LEAVE,
                        outcome: Outcome::Win,
                    };
                } else {
                    self.battle.phase = Phase::AfterPlayer { timer: BEAT };
                }
            }
            1 => {
                self.battle.guarding = true;
                self.push_log(format!("{} guards.", self.player.name));
                self.battle.phase = Phase::AfterPlayer { timer: BEAT };
            }
            _ => {
                self.push_log(format!("{} fled.", self.player.name));
                self.battle.phase = Phase::Leave {
                    timer: LEAVE,
                    outcome: Outcome::Flee,
                };
            }
        }
    }

    fn enemy_turn(&mut self) -> Phase {
        let guard = self.battle.guarding;
        self.battle.guarding = false;
        let dmg = harm(self.monster.atk, self.player.def, guard);
        self.player.hp = (self.player.hp - dmg).max(0);
        self.push_log(format!("{} hits for {dmg}.", self.monster.name));
        if self.player.hp == 0 {
            self.push_log(format!("{} falls.", self.player.name));
            Phase::Leave {
                timer: LEAVE,
                outcome: Outcome::Lose,
            }
        } else {
            Phase::AfterEnemy { timer: BEAT }
        }
    }

    fn apply_outcome(&mut self, outcome: Outcome) {
        match outcome {
            Outcome::Win => self.monster_alive = false,
            Outcome::Flee => {}
            Outcome::Lose => self.player.hp = self.player.max_hp,
        }
        self.queued = None;
        self.walk = None;
        self.mode = Mode::Field;
    }

    fn push_log(&mut self, line: impl Into<String>) {
        self.battle.log.push(line.into());
        let extra = self.battle.log.len().saturating_sub(3);
        if extra > 0 {
            self.battle.log.drain(0..extra);
        }
    }

    fn halt(&mut self) {
        self.walk = None;
        self.queued = None;
    }

    fn settings_path(&self) -> PathBuf {
        self.save_dir.join("settings.txt")
    }

    fn save_path(&self, id: u32) -> PathBuf {
        self.save_dir.join(format!("save-{id:03}.txt"))
    }

    fn write_settings(&self) -> io::Result<()> {
        let text = format!(
            "volume {}\nwidth {}\nheight {}\n",
            self.volume, self.window_w, self.window_h
        );
        fs::write(self.settings_path(), text)
    }

    fn load_settings(&mut self) {
        let Ok(text) = fs::read_to_string(self.settings_path()) else {
            return;
        };
        let mut volume = None;
        let mut width = None;
        let mut height = None;
        for line in text.lines() {
            let Some((key, value)) = line.trim().split_once(' ') else {
                continue;
            };
            match key {
                "volume" => volume = value.parse::<u8>().ok(),
                "width" => width = value.parse::<u32>().ok(),
                "height" => height = value.parse::<u32>().ok(),
                _ => {}
            }
        }
        if let Some(volume) = volume {
            self.volume = volume.min(10);
        }
        if let (Some(width), Some(height)) = (width, height) {
            if (320..=7680).contains(&width) && (240..=4320).contains(&height) {
                self.window_w = width;
                self.window_h = height;
            }
        }
    }

    fn list_saves(&self) -> Vec<u32> {
        let mut ids = Vec::new();
        let Ok(entries) = fs::read_dir(&self.save_dir) else {
            return ids;
        };
        for entry in entries.flatten() {
            let name = entry.file_name();
            let Some(name) = name.to_str() else {
                continue;
            };
            let Some(rest) = name.strip_prefix("save-") else {
                continue;
            };
            let Some(num) = rest.strip_suffix(".txt") else {
                continue;
            };
            if let Ok(id) = num.parse::<u32>() {
                if id > 0 {
                    ids.push(id);
                }
            }
        }
        ids.sort_unstable_by(|a, b| b.cmp(a));
        ids.dedup();
        ids
    }

    fn write_save(&mut self) -> io::Result<u32> {
        let id = self
            .list_saves()
            .into_iter()
            .max()
            .unwrap_or(0)
            .saturating_add(1);
        fs::write(self.save_path(id), self.save_text())?;
        let _ = self.write_settings();
        Ok(id)
    }

    fn save_text(&self) -> String {
        let mut out = String::new();
        let _ = writeln!(out, "fablespace-save 1");
        let _ = writeln!(out, "px {}", self.px);
        let _ = writeln!(out, "py {}", self.py);
        let _ = writeln!(out, "hp {}", self.player.hp);
        let _ = writeln!(out, "max_hp {}", self.player.max_hp);
        let _ = writeln!(
            out,
            "slime_alive {}",
            if self.monster_alive { 1 } else { 0 }
        );
        let _ = writeln!(out, "slime_hp {}", self.monster.hp);
        let _ = writeln!(out, "slime_max {}", self.monster.max_hp);
        let _ = writeln!(out, "slime_x {}", self.mx);
        let _ = writeln!(out, "slime_y {}", self.my);
        let _ = writeln!(out, "volume {}", self.volume);
        let _ = writeln!(out, "width {}", self.window_w);
        let _ = writeln!(out, "height {}", self.window_h);
        let _ = writeln!(out, "characters {}", self.characters.len());
        for character in &self.characters {
            let _ = writeln!(out, "character {}", character.name);
            let _ = writeln!(out, "weapon {}", character.weapon_name);
            let _ = writeln!(out, "weapon_atk {}", character.weapon_atk);
            let _ = writeln!(out, "items {}", character.inventory.len());
            for item in &character.inventory {
                let _ = writeln!(out, "item {}", item.name);
            }
            let _ = writeln!(out, "skills {}", character.skills.len());
            for skill in &character.skills {
                let _ = writeln!(out, "skill {}", skill);
            }
        }
        out
    }

    fn load_newest(&mut self) -> bool {
        for id in self.list_saves() {
            if self.load_id(id) {
                return true;
            }
        }
        false
    }

    fn load_id(&mut self, id: u32) -> bool {
        let Ok(text) = fs::read_to_string(self.save_path(id)) else {
            return false;
        };
        self.apply_save(&text)
    }

    fn apply_save(&mut self, text: &str) -> bool {
        let Some(data) = parse_save(text) else {
            return false;
        };
        let (
            Some(mut px),
            Some(mut py),
            Some(hp),
            Some(max_hp),
            Some(slime_alive),
            Some(slime_hp),
            Some(slime_max),
            Some(mut sx),
            Some(mut sy),
        ) = (
            data.px,
            data.py,
            data.hp,
            data.max_hp,
            data.slime_alive,
            data.slime_hp,
            data.slime_max,
            data.slime_x,
            data.slime_y,
        )
        else {
            return false;
        };
        if data.characters.is_empty() {
            return false;
        }
        if !ground_at(px, py) {
            px = START_X;
            py = START_Y;
        }
        if !ground_at(sx, sy) {
            sx = SLIME_X;
            sy = SLIME_Y;
        }
        let alive = slime_alive && slime_hp > 0;
        if alive && px == sx && py == sy {
            py += 1;
            if !ground_at(px, py) {
                px = START_X;
                py = START_Y;
            }
        }
        let max_hp = max_hp.max(1);
        self.px = px;
        self.py = py;
        self.player.hp = hp.clamp(0, max_hp);
        self.player.max_hp = max_hp;
        self.monster_alive = alive;
        self.monster.hp = slime_hp.max(0);
        self.monster.max_hp = slime_max.max(1);
        self.mx = sx;
        self.my = sy;
        if let Some(volume) = data.volume {
            self.volume = volume.min(10);
        }
        if let (Some(width), Some(height)) = (data.width, data.height) {
            if (320..=7680).contains(&width) && (240..=4320).contains(&height) {
                self.window_w = width;
                self.window_h = height;
            }
        }
        self.characters = data.characters;
        self.party_index = 0;
        self.menu_cursor = 0;
        self.notice = None;
        self.halt();
        self.battle = Battle::fresh(self.monster.name);
        self.mode = Mode::Field;
        let _ = self.write_settings();
        true
    }
}

impl Battle {
    fn fresh(monster: &str) -> Self {
        Self {
            cursor: 0,
            log: vec![format!("{monster} draws near.")],
            phase: Phase::Choose,
            guarding: false,
        }
    }
}

fn mira() -> Character {
    Character {
        name: "Mira".to_string(),
        weapon_name: WEAPON_NAME.to_string(),
        weapon_atk: WEAPON_ATK,
        inventory: Vec::new(),
        skills: Vec::new(),
    }
}

fn mode_of(back: Back) -> Mode {
    match back {
        Back::Field => Mode::Field,
        Back::Talk => Mode::Talk,
        Back::Battle => Mode::Battle,
    }
}

fn default_save_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|dir| dir.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

fn scroll_axis(focus: i32, span: i32, view: i32) -> i32 {
    if span <= view {
        -(view - span) / 2
    } else {
        (focus - view / 2).clamp(0, span - view)
    }
}

fn harm(atk: i32, def: i32, guard: bool) -> i32 {
    let hit = (atk - def).max(1);
    if guard {
        (hit / 2).max(1)
    } else {
        hit
    }
}

fn parse_save(text: &str) -> Option<SaveData> {
    let lines: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();
    if lines.first().copied() != Some("fablespace-save 1") {
        return None;
    }
    let mut data = SaveData {
        px: None,
        py: None,
        hp: None,
        max_hp: None,
        slime_alive: None,
        slime_hp: None,
        slime_max: None,
        slime_x: None,
        slime_y: None,
        volume: None,
        width: None,
        height: None,
        characters: Vec::new(),
    };
    let mut index = 1;
    while index < lines.len() {
        let line = lines[index];
        index += 1;
        if let Some(rest) = line.strip_prefix("characters ") {
            let count: usize = rest.parse().ok()?;
            for _ in 0..count {
                data.characters.push(parse_character(&lines, &mut index)?);
            }
            continue;
        }
        let (key, value) = line.split_once(' ')?;
        match key {
            "px" => data.px = Some(value.parse().ok()?),
            "py" => data.py = Some(value.parse().ok()?),
            "hp" => data.hp = Some(value.parse().ok()?),
            "max_hp" => data.max_hp = Some(value.parse().ok()?),
            "slime_alive" => data.slime_alive = Some(value != "0"),
            "slime_hp" => data.slime_hp = Some(value.parse().ok()?),
            "slime_max" => data.slime_max = Some(value.parse().ok()?),
            "slime_x" => data.slime_x = Some(value.parse().ok()?),
            "slime_y" => data.slime_y = Some(value.parse().ok()?),
            "volume" => data.volume = Some(value.parse().ok()?),
            "width" => data.width = Some(value.parse().ok()?),
            "height" => data.height = Some(value.parse().ok()?),
            _ => return None,
        }
    }
    Some(data)
}

fn parse_character(lines: &[&str], index: &mut usize) -> Option<Character> {
    let name = lines.get(*index)?.strip_prefix("character ")?.to_string();
    *index += 1;
    if name.is_empty() {
        return None;
    }
    let weapon_name = lines.get(*index)?.strip_prefix("weapon ")?.to_string();
    *index += 1;
    if weapon_name.is_empty() {
        return None;
    }
    let weapon_atk = lines
        .get(*index)?
        .strip_prefix("weapon_atk ")?
        .parse()
        .ok()?;
    *index += 1;
    let item_count: usize = lines.get(*index)?.strip_prefix("items ")?.parse().ok()?;
    *index += 1;
    let mut inventory = Vec::new();
    for _ in 0..item_count {
        let item = lines.get(*index)?.strip_prefix("item ")?;
        *index += 1;
        if item.is_empty() {
            return None;
        }
        inventory.push(Item {
            name: item.to_string(),
        });
    }
    let skill_count: usize = lines.get(*index)?.strip_prefix("skills ")?.parse().ok()?;
    *index += 1;
    let mut skills = Vec::new();
    for _ in 0..skill_count {
        let skill = lines.get(*index)?.strip_prefix("skill ")?;
        *index += 1;
        if skill.is_empty() {
            return None;
        }
        skills.push(skill.to_string());
    }
    Some(Character {
        name,
        weapon_name,
        weapon_atk,
        inventory,
        skills,
    })
}

#[cfg(test)]
pub fn isolated_dir() -> PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "fablespace-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[cfg(test)]
mod tests {
    use super::*;

    fn confirm() -> Input {
        Input {
            confirm: true,
            ..Input::default()
        }
    }

    fn down() -> Input {
        Input {
            tap_y: 1,
            ..Input::default()
        }
    }

    fn pause() -> Input {
        Input {
            pause: true,
            ..Input::default()
        }
    }

    fn field_game() -> Game {
        let mut game = Game::with_save_dir(isolated_dir());
        game.start_new_game();
        game
    }

    fn idle(game: &mut Game, frames: u16) {
        for _ in 0..frames {
            game.update(&Input::default());
        }
    }

    fn stand_next_to_slime(game: &mut Game) {
        game.px = game.mx;
        game.py = game.my + 1;
        game.walk = None;
    }

    fn enter_battle(game: &mut Game) {
        stand_next_to_slime(game);
        game.update(&confirm());
        assert_eq!(game.mode, Mode::Talk);
        game.update(&confirm());
        assert!(matches!(game.mode, Mode::Transition { .. }));
        idle(game, TRANSITION_FRAMES + 2);
        assert_eq!(game.mode, Mode::Battle);
        assert!(matches!(game.battle.phase, Phase::Choose));
    }

    fn can_reach(from: (i32, i32), to: (i32, i32)) -> bool {
        let mut seen = vec![false; (MAP_W * MAP_H) as usize];
        let mut queue = std::collections::VecDeque::new();
        if !ground_at(from.0, from.1) {
            return false;
        }
        queue.push_back(from);
        seen[(from.1 * MAP_W + from.0) as usize] = true;
        while let Some((x, y)) = queue.pop_front() {
            if (x, y) == to {
                return true;
            }
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let nx = x + dx;
                let ny = y + dy;
                if !ground_at(nx, ny) {
                    continue;
                }
                let index = (ny * MAP_W + nx) as usize;
                if seen[index] {
                    continue;
                }
                seen[index] = true;
                queue.push_back((nx, ny));
            }
        }
        false
    }

    #[test]
    fn tiles_are_32px_and_the_view_is_about_sixty_by_thirty_three() {
        assert_eq!(SCALE, 2);
        assert_eq!(TILE * SCALE, 32);
        assert_eq!(WIDTH / (TILE * SCALE), 60);
        assert_eq!(HEIGHT / (TILE * SCALE), 33);
        assert_eq!(MAP_W, 120);
        assert_eq!(MAP_H, 70);
        assert!(MAP_W > WIDTH / (TILE * SCALE));
        assert!(MAP_H > HEIGHT / (TILE * SCALE));
    }

    #[test]
    fn camera_follows_the_player_and_clamps() {
        let mut game = field_game();
        assert_eq!(game.camera(), (336, 756));
        let (x, y) = game.camera();
        assert!(x > 0 && y > 0);
        assert!(x + WIDTH < MAP_W * TILE * SCALE);
        assert!(y + HEIGHT < MAP_H * TILE * SCALE);

        game.px = 0;
        game.py = 0;
        game.walk = None;
        assert_eq!(game.camera(), (0, 0));

        game.px = MAP_W - 1;
        game.py = MAP_H - 1;
        let span_x = MAP_W * TILE * SCALE;
        let span_y = MAP_H * TILE * SCALE;
        assert_eq!(game.camera(), (span_x - WIDTH, span_y - HEIGHT));
    }

    #[test]
    fn the_town_is_east_and_the_slime_is_reachable() {
        assert!(ROAD_X > MAP_W / 2);
        assert_eq!(BUILDINGS.len(), 2);
        assert_eq!(BUILDINGS[0].y, BUILDINGS[1].y);
        assert!(BUILDINGS[1].x >= BUILDINGS[0].x + BUILDINGS[0].w);
        assert!(BUILDINGS[1].x - (BUILDINGS[0].x + BUILDINGS[0].w) <= 2);
        assert_eq!(ROAD_Y0, BUILDINGS[0].y + BUILDINGS[0].h);
        assert!(BUILDINGS.iter().all(|building| building.x > MAP_W / 2));
        for building in BUILDINGS {
            assert!(!ground_at(building.x, building.y));
            assert!(!ground_at(building.x, building.y + building.roof_rows));
        }
        assert!(!ground_at(-1, 0));
        assert!(!ground_at(MAP_W, MAP_H - 1));
        assert!(ground_at(ROAD_X + 1, ROAD_Y0));
        assert!(can_reach((START_X, START_Y), (SLIME_X, SLIME_Y + 1)));
        assert!(can_reach((START_X, START_Y), (ROAD_X + 1, 30)));
    }

    #[test]
    fn a_step_into_a_building_does_nothing() {
        let mut game = field_game();
        let wall_y = BUILDINGS[0].y + BUILDINGS[0].roof_rows;
        game.px = BUILDINGS[0].x - 1;
        game.py = wall_y;
        game.walk = None;
        assert!(game.ground(game.px, game.py));
        game.update(&Input {
            tap_x: 1,
            ..Input::default()
        });
        idle(&mut game, u16::from(WALK_FRAMES) + 2);
        assert_eq!((game.px, game.py), (BUILDINGS[0].x - 1, wall_y));

        game.px = 0;
        game.py = 0;
        game.walk = None;
        game.update(&Input {
            tap_x: -1,
            ..Input::default()
        });
        idle(&mut game, u16::from(WALK_FRAMES) + 2);
        assert_eq!((game.px, game.py), (0, 0));
    }

    #[test]
    fn far_away_confirm_does_nothing() {
        let mut game = field_game();
        game.update(&confirm());
        assert_eq!(game.mode, Mode::Field);
    }

    #[test]
    fn diagonal_is_not_close_enough_to_talk() {
        let mut game = field_game();
        game.px = game.mx + 1;
        game.py = game.my + 1;
        game.update(&confirm());
        assert_eq!(game.mode, Mode::Field);
    }

    #[test]
    fn monster_blocks_the_tile() {
        let mut game = field_game();
        stand_next_to_slime(&mut game);
        game.update(&Input {
            tap_y: -1,
            ..Input::default()
        });
        idle(&mut game, u16::from(WALK_FRAMES) + 2);
        assert_eq!((game.px, game.py), (game.mx, game.my + 1));
    }

    #[test]
    fn a_step_lands_on_the_next_tile() {
        let mut game = field_game();
        let start = game.px;
        game.update(&Input {
            tap_x: 1,
            ..Input::default()
        });
        assert!(game.walk.is_some());
        idle(&mut game, u16::from(WALK_FRAMES));
        assert_eq!(game.px, start + 1);
        assert!(game.walk.is_none());
    }

    #[test]
    fn pause_opens_and_closes_without_moving() {
        let mut game = field_game();
        let (px, py) = (game.px, game.py);
        game.update(&pause());
        assert_eq!(game.mode, Mode::System(Back::Field));
        idle(&mut game, 10);
        assert_eq!((game.px, game.py), (px, py));
        game.update(&pause());
        assert_eq!(game.mode, Mode::Field);
    }

    #[test]
    fn continue_without_a_save_stays_on_the_title() {
        let mut game = Game::with_save_dir(isolated_dir());
        assert_eq!(game.mode, Mode::Title);
        game.update(&confirm());
        assert_eq!(game.mode, Mode::Title);
        assert_eq!(game.notice, Some("No save."));
    }

    #[test]
    fn title_quit_closes_immediately() {
        let dir = isolated_dir();
        let mut game = Game::with_save_dir(&dir);
        game.menu_cursor = 3;
        game.update(&confirm());
        assert!(game.should_quit());
        assert!(game.list_saves().is_empty());
    }

    #[test]
    fn save_keeps_place_slime_inventory_and_settings() {
        let dir = isolated_dir();
        let mut game = Game::with_save_dir(&dir);
        game.start_new_game();
        game.px = ROAD_X + 1;
        game.py = 20;
        game.player.hp = 11;
        game.monster.hp = 4;
        game.volume = 4;
        game.window_w = 1280;
        game.window_h = 720;
        game.characters[0].inventory.push(Item {
            name: "Herb".into(),
        });
        game.characters.push(Character {
            name: "Jun".into(),
            weapon_name: "Bow".into(),
            weapon_atk: 2,
            inventory: vec![Item {
                name: "Rope".into(),
            }],
            skills: vec!["Focus".into()],
        });
        let first = game.write_save().unwrap();
        game.px = 55;
        game.py = 22;
        let second = game.write_save().unwrap();
        assert!(second > first);

        let mut loaded = Game::with_save_dir(&dir);
        loaded.update(&confirm());
        assert_eq!(loaded.mode, Mode::Field);
        assert_eq!((loaded.px, loaded.py), (55, 22));
        assert_eq!(loaded.player.hp, 11);
        assert_eq!(loaded.monster.hp, 4);
        assert!(loaded.monster_alive);
        assert_eq!((loaded.mx, loaded.my), (SLIME_X, SLIME_Y));
        assert_eq!(loaded.volume, 4);
        assert_eq!(loaded.window_size(), (1280, 720));
        assert_eq!(loaded.characters[0].name, "Mira");
        assert_eq!(loaded.characters[0].weapon_name, WEAPON_NAME);
        assert_eq!(loaded.characters[0].inventory[0].name, "Herb");
        assert!(loaded.characters[0].skills.is_empty());
        assert_eq!(loaded.characters[1].inventory[0].name, "Rope");
        assert_eq!(loaded.characters[1].skills[0], "Focus");
        assert_ne!(
            loaded.characters[0].inventory[0].name,
            loaded.characters[1].inventory[0].name
        );

        let mut picker = Game::with_save_dir(&dir);
        picker.menu_cursor = 1;
        picker.update(&confirm());
        assert!(matches!(
            picker.mode,
            Mode::LoadList {
                from_title: true,
                ..
            }
        ));
        assert_eq!(picker.slots, vec![second, first]);
        picker.update(&down());
        picker.update(&confirm());
        assert_eq!((picker.px, picker.py), (ROAD_X + 1, 20));

        let mut fresh = Game::with_save_dir(&dir);
        fresh.menu_cursor = 2;
        fresh.update(&confirm());
        assert_eq!(fresh.mode, Mode::Field);
        assert_eq!((fresh.px, fresh.py), (START_X, START_Y));
        assert_eq!(fresh.characters.len(), 1);
        assert!(fresh.characters[0].inventory.is_empty());
        assert_eq!(fresh.volume, 4);
        assert_eq!(fresh.player.hp, fresh.player.max_hp);
        assert!(fresh.monster_alive);
        assert_eq!(fresh.monster.hp, fresh.monster.max_hp);
    }

    #[test]
    fn audio_and_resolution_apply_before_any_save() {
        let dir = isolated_dir();
        let mut game = Game::with_save_dir(&dir);
        game.start_new_game();
        game.update(&pause());
        game.update(&down());
        game.update(&down());
        assert_eq!(game.menu_cursor, AUDIO_ITEM);
        game.update(&Input {
            tap_x: -1,
            ..Input::default()
        });
        assert_eq!(game.volume, 9);
        assert_eq!((game.master_gain() * 10.0).round() as u8, 9);
        game.update(&down());
        assert_eq!(game.menu_cursor, RESOLUTION_ITEM);
        game.update(&Input {
            tap_x: 1,
            ..Input::default()
        });
        assert_eq!(game.window_size(), (2560, 1440));
        assert!(game.list_saves().is_empty());
        let text = fs::read_to_string(dir.join("settings.txt")).unwrap();
        assert!(text.contains("volume 9"));
        assert!(text.contains("width 2560"));
        assert!(text.contains("height 1440"));
        let again = Game::with_save_dir(&dir);
        assert_eq!(again.mode, Mode::Title);
        assert_eq!(again.volume, 9);
        assert_eq!(again.window_size(), (2560, 1440));
    }

    #[test]
    fn quit_can_close_or_save_first() {
        let dir = isolated_dir();
        let mut game = Game::with_save_dir(&dir);
        game.start_new_game();
        game.px = 44;
        game.update(&pause());
        for _ in 0..QUIT_ITEM {
            game.update(&down());
        }
        game.update(&confirm());
        assert!(matches!(game.mode, Mode::QuitAsk(Back::Field)));
        game.menu_cursor = 1;
        game.update(&confirm());
        assert!(game.should_quit());
        assert!(game.list_saves().is_empty());

        let mut game = Game::with_save_dir(&dir);
        game.start_new_game();
        game.px = 44;
        game.update(&pause());
        for _ in 0..SAVE_QUIT_ITEM {
            game.update(&down());
        }
        game.update(&confirm());
        assert!(game.should_quit());
        assert_eq!(game.list_saves(), vec![1]);

        let mut game = Game::with_save_dir(&dir);
        game.start_new_game();
        game.px = 46;
        game.update(&pause());
        for _ in 0..QUIT_ITEM {
            game.update(&down());
        }
        game.update(&confirm());
        assert!(matches!(game.mode, Mode::QuitAsk(_)));
        game.update(&confirm());
        assert!(game.should_quit());
        let mut loaded = Game::with_save_dir(&dir);
        loaded.update(&confirm());
        assert_eq!(loaded.px, 46);
    }

    #[test]
    fn tab_opens_the_party_and_its_lists() {
        let mut game = field_game();
        assert_eq!(game.characters.len(), 1);
        assert_eq!(game.characters[0].name, "Mira");
        assert_eq!(game.characters[0].weapon_name, WEAPON_NAME);
        assert_eq!(game.characters[0].weapon_atk, WEAPON_ATK);
        assert!(game.characters[0].skills.is_empty());
        assert!(game.characters[0].inventory.is_empty());
        let (px, py) = (game.px, game.py);
        game.update(&Input {
            tab: true,
            ..Input::default()
        });
        assert_eq!(game.mode, Mode::Party);
        idle(&mut game, 8);
        assert_eq!((game.px, game.py), (px, py));
        game.update(&down());
        game.update(&confirm());
        assert_eq!(game.mode, Mode::Skills);
        game.update(&pause());
        assert_eq!(game.mode, Mode::Party);
        game.update(&down());
        game.update(&confirm());
        assert_eq!(game.mode, Mode::Inventory);
        game.update(&Input {
            tab: true,
            ..Input::default()
        });
        assert_eq!(game.mode, Mode::Field);
    }

    #[test]
    fn attack_twice_wins_and_removes_the_monster() {
        let mut game = field_game();
        enter_battle(&mut game);
        for _ in 0..500 {
            if game.mode == Mode::Field {
                break;
            }
            if matches!(game.battle.phase, Phase::Choose) {
                game.update(&confirm());
            } else {
                game.update(&Input::default());
            }
        }
        assert_eq!(game.mode, Mode::Field);
        assert!(!game.monster_alive);
        assert_eq!(game.monster.hp, 0);
        game.update(&confirm());
        assert_eq!(game.mode, Mode::Field);
    }

    #[test]
    fn guard_halves_the_counterattack() {
        let mut game = field_game();
        enter_battle(&mut game);
        game.update(&Input {
            tap_y: 1,
            ..Input::default()
        });
        assert_eq!(game.battle.cursor, 1);
        let hp = game.player.hp;
        game.update(&confirm());
        idle(&mut game, u16::from(BEAT));
        // Unguarded slime damage is 5 - 2 = 3, so a guard takes 1.
        assert_eq!(game.player.hp, hp - 1);
        assert!(!game.battle.guarding);
    }

    #[test]
    fn flee_leaves_the_monster_and_keeps_hp() {
        let mut game = field_game();
        enter_battle(&mut game);
        game.update(&confirm());
        idle(&mut game, u16::from(BEAT) * 2 + 4);
        assert!(matches!(game.battle.phase, Phase::Choose));
        assert!(game.monster.hp < game.monster.max_hp);
        assert!(game.player.hp < game.player.max_hp);
        let (player_hp, monster_hp) = (game.player.hp, game.monster.hp);
        game.battle.cursor = 2;
        game.update(&confirm());
        idle(&mut game, u16::from(LEAVE) + 2);
        assert_eq!(game.mode, Mode::Field);
        assert!(game.monster_alive);
        assert_eq!(game.player.hp, player_hp);
        assert_eq!(game.monster.hp, monster_hp);
    }

    #[test]
    fn loss_restores_the_player_and_keeps_the_monster() {
        let mut game = field_game();
        enter_battle(&mut game);
        game.player.hp = 3;
        game.monster.hp = 8;
        game.update(&confirm());
        idle(&mut game, u16::from(BEAT) + u16::from(LEAVE) + 4);
        assert_eq!(game.mode, Mode::Field);
        assert!(game.monster_alive);
        assert_eq!(game.player.hp, game.player.max_hp);
        assert_eq!(game.monster.hp, 2);
    }
}
