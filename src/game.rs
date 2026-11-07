//! Walk a field, talk, and fight.
//!
//! The 120x70 map, the town, Mira, and the slime's branching talk come from
//! the project files. Esc is the system menu. Tab is the party menu. Edit
//! mode is only opened from the title.

use crate::atlas::TILE;
use crate::project::import_png;
#[cfg(test)]
use crate::project::{default_project, Building};
pub use crate::project::{
    BattleLink, Character, Choice, Item, Need, Node, Project, Terrain, MAP_H, MAP_W,
};
use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const WIDTH: i32 = 1920;
pub const HEIGHT: i32 = 1080;
/// Field tiles and sprites are drawn at this multiple of the 16px art.
/// 2x is 32px per tile, so the 1920x1080 view shows 60 x 33 tiles.
pub const SCALE: i32 = 2;
pub const WALK_FRAMES: u8 = 8;
pub const TRANSITION_FRAMES: u16 = 36;
pub const BEAT: u8 = 28;
pub const LEAVE: u8 = 42;

#[cfg(test)]
pub const TALK_LINE: &str = "Keep off my grass.";
pub const COMMANDS: [&str; 3] = ["Attack", "Guard", "Flee"];
#[cfg(test)]
pub const WEAPON_NAME: &str = "Short Sword";
#[cfg(test)]
pub const WEAPON_ATK: i32 = 3;

#[cfg(test)]
pub const START_X: i32 = 40;
#[cfg(test)]
pub const START_Y: i32 = 40;
#[cfg(test)]
pub const SLIME_X: i32 = 48;
#[cfg(test)]
pub const SLIME_Y: i32 = 35;

#[cfg(test)]
pub const ROAD_X: i32 = 99;
#[cfg(test)]
pub const ROAD_W: i32 = 3;
#[cfg(test)]
pub const ROAD_Y0: i32 = 14;
#[cfg(test)]
pub const ROAD_Y1: i32 = 62;

pub const TITLE_ITEMS: [&str; 5] = ["Continue", "Load Game", "New Game", "Edit", "Quit"];
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
const EDIT_ITEM: usize = 3;

pub const RESOLUTIONS: [(u32, u32); 4] = [(1280, 720), (1600, 900), (1920, 1080), (2560, 1440)];

#[cfg(test)]
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

#[cfg(test)]
pub const MUSHROOMS: [(i32, i32); 4] = [(22, 38), (30, 32), (55, 44), (36, 50)];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditPage {
    Map,
    Tiles,
    Characters,
    Dialogue,
}

impl EditPage {
    pub const ALL: [EditPage; 4] = [
        EditPage::Map,
        EditPage::Tiles,
        EditPage::Characters,
        EditPage::Dialogue,
    ];

    fn next(self) -> Self {
        Self::ALL[(self.index() + 1) % Self::ALL.len()]
    }

    fn prev(self) -> Self {
        Self::ALL[(self.index() + Self::ALL.len() - 1) % Self::ALL.len()]
    }

    pub fn index(self) -> usize {
        Self::ALL.iter().position(|page| *page == self).unwrap_or(0)
    }

    pub fn name(self) -> &'static str {
        match self {
            EditPage::Map => "Map",
            EditPage::Tiles => "Tiles",
            EditPage::Characters => "Characters",
            EditPage::Dialogue => "Dialogue",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Brush {
    Grass,
    Road,
    WallWood,
    WallStone,
    HouseWood,
    HouseStone,
    Void,
    Custom(u16),
}

impl Brush {
    pub const PALETTE: [Brush; 7] = [
        Brush::Grass,
        Brush::Road,
        Brush::WallWood,
        Brush::WallStone,
        Brush::HouseWood,
        Brush::HouseStone,
        Brush::Void,
    ];

    pub fn from_digit(n: u8) -> Option<Self> {
        Some(match n {
            1 => Brush::Grass,
            2 => Brush::Road,
            3 => Brush::WallWood,
            4 => Brush::WallStone,
            5 => Brush::HouseWood,
            6 => Brush::HouseStone,
            7 => Brush::Void,
            _ => return None,
        })
    }

    pub fn label(self) -> &'static str {
        match self {
            Brush::Grass => "1 Grass",
            Brush::Road => "2 Road",
            Brush::WallWood => "3 Wood wall",
            Brush::WallStone => "4 Stone wall",
            Brush::HouseWood => "5 Wood house",
            Brush::HouseStone => "6 Stone house",
            Brush::Void => "7 Void",
            Brush::Custom(_) => "Imported tile",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TypeTarget {
    None,
    Weapon,
    Require(usize),
    SetFlag(usize, usize),
    BattleEntity,
    BattleMap,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiaRow {
    Node,
    Require(usize),
    AddRequire,
    Choice(usize),
    SetFlag(usize, usize),
    AddSet(usize),
    AddChoice,
    ToggleBattle,
    BattleEntity,
    BattleMap,
    AddNode,
}

#[derive(Clone, Debug)]
pub struct EditState {
    pub page: EditPage,
    pub brush: Brush,
    pub cx: i32,
    pub cy: i32,
    pub row: usize,
    pub tile_path: String,
    pub char_index: usize,
    pub node_index: usize,
    pub custom: Option<u16>,
    type_target: TypeTarget,
}

impl Default for EditState {
    fn default() -> Self {
        Self {
            page: EditPage::Map,
            brush: Brush::Grass,
            cx: 96,
            cy: 9,
            row: 0,
            tile_path: String::new(),
            char_index: 0,
            node_index: 0,
            custom: None,
            type_target: TypeTarget::None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Fighter {
    pub name: String,
    pub hp: i32,
    pub max_hp: i32,
    pub atk: i32,
    pub def: i32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Input {
    pub held_x: i32,
    pub held_y: i32,
    pub tap_x: i32,
    pub tap_y: i32,
    pub confirm: bool,
    pub pause: bool,
    pub tab: bool,
    pub backspace: bool,
    pub page_left: bool,
    pub page_right: bool,
    pub text: Option<char>,
    pub digit: Option<u8>,
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
    SaveAsk(Back),
    NameSlot(Back),
    Party,
    Skills,
    Inventory,
    Edit,
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
    name: Option<String>,
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
    flags: BTreeSet<String>,
    characters: Vec<Character>,
}

struct SlotHead {
    name: String,
    stamp: u64,
    path: PathBuf,
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
    pub notice: Option<String>,
    pub characters: Vec<Character>,
    pub slots: Vec<String>,
    pub project: Project,
    pub flags: BTreeSet<String>,
    pub slot_name: Option<String>,
    pub talk_node: usize,
    pub talk_choice: usize,
    pub edit: EditState,
    pub name_buf: String,
    walk: Option<Walk>,
    queued: Option<(i32, i32)>,
    quit: bool,
    save_dir: PathBuf,
    project_dir: Option<PathBuf>,
}

#[cfg(test)]
pub fn terrain_at(x: i32, y: i32) -> Option<Terrain> {
    default_project().terrain(x, y)
}

#[cfg(test)]
pub fn ground_at(x: i32, y: i32) -> bool {
    default_project().walkable(x, y)
}

impl Game {
    pub fn new() -> Self {
        let (project, dir) = Project::load();
        Self::boot(default_save_dir(), dir, project)
    }

    #[cfg(test)]
    pub fn with_save_dir(dir: impl Into<PathBuf>) -> Self {
        Self::boot(dir.into(), None, Project::embedded())
    }

    pub fn start_new_game(&mut self) {
        let project = self.project.clone();
        let volume = self.volume;
        let window_w = self.window_w;
        let window_h = self.window_h;
        let save_dir = self.save_dir.clone();
        let project_dir = self.project_dir.clone();
        *self = Self::from_project(save_dir, project_dir, project);
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
            Mode::SaveAsk(_) => self.update_save_ask(input),
            Mode::NameSlot(_) => self.update_name(input),
            Mode::Party => self.update_party(input),
            Mode::Skills | Mode::Inventory => self.update_submenu(input),
            Mode::Edit => self.update_edit(input),
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

    pub fn ground(&self, x: i32, y: i32) -> bool {
        self.project.walkable(x, y)
    }

    pub fn current_line(&self) -> &str {
        self.project
            .talk
            .nodes
            .get(self.talk_node)
            .map(|node| node.text.as_str())
            .unwrap_or("")
    }

    pub fn current_choices(&self) -> &[Choice] {
        self.project
            .talk
            .nodes
            .get(self.talk_node)
            .map(|node| node.choices.as_slice())
            .unwrap_or(&[])
    }

    pub fn dialogue_rows(&self) -> Vec<DiaRow> {
        let Some(node) = self.project.talk.nodes.get(self.edit.node_index) else {
            return vec![DiaRow::AddNode];
        };
        let mut rows = vec![DiaRow::Node];
        for index in 0..node.require.len() {
            rows.push(DiaRow::Require(index));
        }
        rows.push(DiaRow::AddRequire);
        for index in 0..node.choices.len() {
            rows.push(DiaRow::Choice(index));
            for flag in 0..node.choices[index].sets.len() {
                rows.push(DiaRow::SetFlag(index, flag));
            }
            rows.push(DiaRow::AddSet(index));
        }
        rows.push(DiaRow::AddChoice);
        rows.push(DiaRow::ToggleBattle);
        if node.battle.is_some() {
            rows.push(DiaRow::BattleEntity);
            rows.push(DiaRow::BattleMap);
        }
        rows.push(DiaRow::AddNode);
        rows
    }

    pub fn import_tiles(&mut self, path: &Path) -> Result<i32, String> {
        let sheet = import_png(path)?;
        let count = sheet.count;
        self.project.sheet = sheet;
        self.project.dirty = true;
        if count > 0 {
            self.edit.custom = Some(0);
            self.edit.brush = Brush::Custom(0);
        }
        Ok(count)
    }

    fn boot(save_dir: PathBuf, project_dir: Option<PathBuf>, project: Project) -> Self {
        let mut game = Self::from_project(save_dir, project_dir, project);
        game.load_settings();
        game
    }

    fn from_project(save_dir: PathBuf, project_dir: Option<PathBuf>, project: Project) -> Self {
        let lead = project
            .party
            .first()
            .cloned()
            .unwrap_or_else(Character::mira);
        let characters = if project.party.is_empty() {
            vec![lead.clone()]
        } else {
            project.party.clone()
        };
        let lead = characters[0].clone();
        Self {
            mode: Mode::Title,
            player: Fighter {
                name: lead.name.clone(),
                hp: lead.hp,
                max_hp: lead.max_hp.max(1),
                atk: lead.atk,
                def: lead.def,
            },
            monster: Fighter {
                name: project.slime_name.clone(),
                hp: project.slime_hp,
                max_hp: project.slime_max.max(1),
                atk: project.slime_atk,
                def: project.slime_def,
            },
            monster_alive: true,
            px: project.start_x,
            py: project.start_y,
            mx: project.slime_x,
            my: project.slime_y,
            battle: Battle::fresh(&project.slime_name),
            menu_cursor: 0,
            party_index: 0,
            volume: 10,
            window_w: WIDTH as u32,
            window_h: HEIGHT as u32,
            notice: None,
            characters,
            slots: Vec::new(),
            project,
            flags: BTreeSet::new(),
            slot_name: None,
            talk_node: 0,
            talk_choice: 0,
            edit: EditState::default(),
            name_buf: String::new(),
            walk: None,
            queued: None,
            quit: false,
            save_dir,
            project_dir,
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
                    self.notice = Some("No save.".into());
                }
            }
            1 => self.open_load(true, Back::Field),
            2 => self.start_new_game(),
            EDIT_ITEM => {
                self.edit = EditState::default();
                self.notice = None;
                self.mode = Mode::Edit;
            }
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
            let name = self.slots[index].clone();
            if !self.load_name(&name) {
                self.notice = Some("Save failed.".into());
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
                self.menu_cursor = 0;
                self.notice = None;
                self.mode = Mode::SaveAsk(back);
            }
            LOAD_ITEM => self.open_load(false, back),
            SAVE_QUIT_ITEM => {
                if self.overwrite_current().is_ok() {
                    self.quit = true;
                } else {
                    self.notice = Some("Save failed.".into());
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
            if self.overwrite_current().is_ok() {
                self.quit = true;
            } else {
                self.notice = Some("Save failed.".into());
            }
        } else {
            self.quit = true;
        }
    }

    fn update_save_ask(&mut self, input: &Input) {
        let back = match self.mode {
            Mode::SaveAsk(back) => back,
            _ => return,
        };
        if input.pause {
            self.menu_cursor = SAVE_ITEM;
            self.notice = None;
            self.mode = Mode::System(back);
            return;
        }
        if input.tap_y != 0 {
            self.notice = None;
            self.step_cursor(input.tap_y, 2);
        }
        if !input.confirm {
            return;
        }
        if self.menu_cursor == 0 {
            self.notice = match self.overwrite_current() {
                Ok(_) => Some("Saved.".into()),
                Err(_) => Some("Save failed.".into()),
            };
            self.menu_cursor = SAVE_ITEM;
            self.mode = Mode::System(back);
        } else {
            self.name_buf = match &self.slot_name {
                Some(name) => format!("{name} fork"),
                None => "Fork".into(),
            };
            self.notice = None;
            self.mode = Mode::NameSlot(back);
        }
    }

    fn update_name(&mut self, input: &Input) {
        let back = match self.mode {
            Mode::NameSlot(back) => back,
            _ => return,
        };
        if input.pause {
            self.notice = None;
            self.mode = Mode::SaveAsk(back);
            return;
        }
        if let Some(ch) = input.text {
            if self.name_buf.chars().count() < 24
                && (ch.is_ascii_alphanumeric() || ch == ' ' || ch == '-')
            {
                self.name_buf.push(ch);
            }
        }
        if input.backspace {
            self.name_buf.pop();
        }
        if !input.confirm {
            return;
        }
        let name = self.name_buf.trim().to_string();
        if name.is_empty() {
            self.notice = Some("Need a name.".into());
            return;
        }
        match self.fork_named(&name) {
            Ok(_) => {
                self.notice = Some("Forked.".into());
                self.menu_cursor = SAVE_ITEM;
                self.mode = Mode::System(back);
            }
            Err(_) => self.notice = Some("Save failed.".into()),
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
                    self.open_talk();
                } else {
                    self.move_player(input);
                }
            }
            Mode::Talk => self.update_talk(input),
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

    fn update_edit(&mut self, input: &Input) {
        if self.edit.type_target != TypeTarget::None {
            self.type_edit(input);
            return;
        }
        if input.pause {
            self.leave_edit();
            return;
        }
        if input.page_right {
            self.edit.page = self.edit.page.next();
            self.edit.row = 0;
            self.notice = None;
            return;
        }
        if input.page_left {
            self.edit.page = self.edit.page.prev();
            self.edit.row = 0;
            self.notice = None;
            return;
        }
        match self.edit.page {
            EditPage::Map => self.edit_map(input),
            EditPage::Tiles => self.edit_tiles(input),
            EditPage::Characters => self.edit_characters(input),
            EditPage::Dialogue => self.edit_dialogue(input),
        }
    }

    fn leave_edit(&mut self) {
        if self.project.dirty {
            if let Some(dir) = self.project_dir.clone() {
                if self.project.write_dir(&dir).is_ok() {
                    self.project.dirty = false;
                }
            }
        }
        self.menu_cursor = EDIT_ITEM;
        self.notice = None;
        self.mode = Mode::Title;
    }

    fn edit_map(&mut self, input: &Input) {
        if let Some(digit) = input.digit {
            if let Some(brush) = Brush::from_digit(digit) {
                self.edit.brush = brush;
            }
        }
        if input.tap_x != 0 || input.tap_y != 0 {
            self.edit.cx = (self.edit.cx + input.tap_x.signum()).clamp(0, MAP_W - 1);
            self.edit.cy = (self.edit.cy + input.tap_y.signum()).clamp(0, MAP_H - 1);
        }
        if input.confirm {
            self.paint();
        }
    }

    fn paint(&mut self) {
        let x = self.edit.cx;
        let y = self.edit.cy;
        match self.edit.brush {
            Brush::Grass => self.project.set_terrain(x, y, Terrain::Grass),
            Brush::Road => self.paint_road(x, y),
            Brush::WallWood => self.project.set_terrain(x, y, Terrain::WallWood),
            Brush::WallStone => self.project.set_terrain(x, y, Terrain::WallStone),
            Brush::HouseWood => self.stamp_house(x, y, true),
            Brush::HouseStone => self.stamp_house(x, y, false),
            Brush::Void => self.project.set_terrain(x, y, Terrain::Void),
            Brush::Custom(id) => self.project.set_terrain(x, y, Terrain::Custom(id)),
        }
    }

    fn paint_road(&mut self, x: i32, y: i32) {
        self.project.set_terrain(x, y, Terrain::Road);
        let mut x0 = x;
        while x0 > 0 && self.project.is_road(x0 - 1, y) {
            x0 -= 1;
        }
        let mut x1 = x;
        while x1 + 1 < MAP_W && self.project.is_road(x1 + 1, y) {
            x1 += 1;
        }
        let len = x1 - x0 + 1;
        for i in 0..len {
            let kind = if len == 1 {
                Terrain::Road
            } else if i == 0 {
                Terrain::RoadLeft
            } else if i + 1 == len {
                Terrain::RoadRight
            } else {
                Terrain::Road
            };
            self.project.set_terrain(x0 + i, y, kind);
        }
    }

    fn stamp_house(&mut self, x: i32, y: i32, wood: bool) {
        for dy in 0..8 {
            for dx in 0..7 {
                let terrain = if dy < 3 {
                    if wood {
                        Terrain::RoofRed
                    } else {
                        Terrain::RoofBlue
                    }
                } else if wood {
                    Terrain::WallWood
                } else {
                    Terrain::WallStone
                };
                self.project.set_terrain(x + dx, y + dy, terrain);
            }
        }
    }

    fn edit_tiles(&mut self, input: &Input) {
        if let Some(ch) = input.text {
            if is_path_char(ch) && self.edit.tile_path.chars().count() < 180 {
                self.edit.tile_path.push(ch);
            }
        }
        if input.backspace {
            self.edit.tile_path.pop();
        }
        if input.tap_x != 0 && self.project.sheet.count > 0 {
            let count = self.project.sheet.count as i32;
            let cur = i32::from(self.edit.custom.unwrap_or(0));
            let next = (cur + input.tap_x.signum()).rem_euclid(count) as u16;
            self.edit.custom = Some(next);
            self.edit.brush = Brush::Custom(next);
        }
        if !input.confirm {
            return;
        }
        let path = self.edit.tile_path.clone();
        match self.import_tiles(Path::new(&path)) {
            Ok(count) => self.notice = Some(format!("{count} tiles")),
            Err(err) => self.notice = Some(err),
        }
    }

    fn edit_characters(&mut self, input: &Input) {
        let len = 6;
        if input.tap_y != 0 {
            self.step_edit_row(input.tap_y, len);
        }
        if self.edit.row == 0 && input.tap_x != 0 && !self.project.party.is_empty() {
            let count = self.project.party.len() as i32;
            let next = self.edit.char_index as i32 + input.tap_x.signum();
            self.edit.char_index = next.rem_euclid(count) as usize;
        }
        if !input.confirm {
            return;
        }
        match self.edit.row {
            1 => self.add_character(),
            2 => self.remove_character(),
            3 => self.edit.type_target = TypeTarget::Weapon,
            4 => self.add_skill(),
            5 => self.add_item(),
            _ => {}
        }
    }

    fn add_character(&mut self) {
        let ally = Character::ally(self.project.party.len());
        self.project.party.push(ally);
        self.project.dirty = true;
        self.edit.char_index = self.project.party.len() - 1;
        self.sync_party();
    }

    fn remove_character(&mut self) {
        if self.project.party.len() == 1 && self.project.party[0].name == "Mira" {
            self.notice = Some("Mira stays.".into());
            return;
        }
        if self.project.party.len() <= 1 {
            self.notice = Some("Need a character.".into());
            return;
        }
        let index = self.edit.char_index.min(self.project.party.len() - 1);
        self.project.party.remove(index);
        self.edit.char_index = self.edit.char_index.min(self.project.party.len() - 1);
        self.project.dirty = true;
        self.notice = None;
        self.sync_party();
    }

    fn add_skill(&mut self) {
        let index = self.edit.char_index.min(self.project.party.len() - 1);
        self.project.party[index].skills.push("New Skill".into());
        self.project.dirty = true;
        self.sync_party();
    }

    fn add_item(&mut self) {
        let index = self.edit.char_index.min(self.project.party.len() - 1);
        self.project.party[index].inventory.push(Item {
            name: "Herb".into(),
        });
        self.project.dirty = true;
        self.sync_party();
    }

    fn sync_party(&mut self) {
        self.characters = self.project.party.clone();
        if self.party_index >= self.characters.len() {
            self.party_index = 0;
        }
    }

    fn edit_dialogue(&mut self, input: &Input) {
        let len = self.dialogue_rows().len();
        if len == 0 {
            return;
        }
        if self.edit.row >= len {
            self.edit.row = 0;
        }
        if input.tap_y != 0 {
            self.step_edit_row(input.tap_y, len);
        }
        let row = self.dialogue_rows()[self.edit.row.min(self.dialogue_rows().len() - 1)];
        if input.tap_x != 0 {
            self.edit_dialogue_side(row, input.tap_x);
            return;
        }
        if input.confirm {
            self.edit_dialogue_confirm(row);
        }
    }

    fn edit_dialogue_side(&mut self, row: DiaRow, tap_x: i32) {
        match row {
            DiaRow::Node => {
                let count = self.project.talk.nodes.len() as i32;
                if count == 0 {
                    return;
                }
                let next = self.edit.node_index as i32 + tap_x.signum();
                self.edit.node_index = next.rem_euclid(count) as usize;
                self.edit.row = 0;
            }
            DiaRow::Require(index) => {
                if let Some(need) = self.project.talk.nodes[self.edit.node_index]
                    .require
                    .get_mut(index)
                {
                    need.want = !need.want;
                    self.project.dirty = true;
                }
            }
            DiaRow::Choice(index) => {
                let dests = self.destinations();
                if dests.is_empty() {
                    return;
                }
                let current = self.project.talk.nodes[self.edit.node_index].choices[index]
                    .goto
                    .clone();
                let pos = dests.iter().position(|id| id == &current).unwrap_or(0);
                let next = if tap_x > 0 {
                    (pos + 1) % dests.len()
                } else {
                    (pos + dests.len() - 1) % dests.len()
                };
                self.project.talk.nodes[self.edit.node_index].choices[index].goto =
                    dests[next].clone();
                self.project.dirty = true;
            }
            _ => {}
        }
    }

    fn edit_dialogue_confirm(&mut self, row: DiaRow) {
        let index = self.edit.node_index;
        match row {
            DiaRow::AddRequire => {
                self.project.talk.nodes[index].require.push(Need {
                    flag: "flag".into(),
                    want: true,
                });
                self.project.dirty = true;
            }
            DiaRow::AddSet(choice) => {
                if let Some(choice) = self.project.talk.nodes[index].choices.get_mut(choice) {
                    choice.sets.push("flag".into());
                    self.project.dirty = true;
                }
            }
            DiaRow::AddChoice => {
                self.project.talk.nodes[index].choices.push(Choice {
                    label: "Next".into(),
                    goto: "end".into(),
                    sets: Vec::new(),
                });
                self.project.dirty = true;
            }
            DiaRow::ToggleBattle => {
                let node = &mut self.project.talk.nodes[index];
                if node.battle.is_some() {
                    node.battle = None;
                } else {
                    node.battle = Some(BattleLink {
                        entity: "slime".into(),
                        map: self.project.map_name.clone(),
                    });
                }
                self.project.dirty = true;
            }
            DiaRow::AddNode => {
                let id = format!("line{}", self.project.talk.nodes.len());
                self.project.talk.nodes.push(Node {
                    id,
                    text: "...".into(),
                    require: Vec::new(),
                    choices: Vec::new(),
                    battle: None,
                });
                self.edit.node_index = self.project.talk.nodes.len() - 1;
                self.edit.row = 0;
                self.project.dirty = true;
            }
            DiaRow::Require(flag) => self.edit.type_target = TypeTarget::Require(flag),
            DiaRow::SetFlag(choice, flag) => {
                self.edit.type_target = TypeTarget::SetFlag(choice, flag);
            }
            DiaRow::BattleEntity => self.edit.type_target = TypeTarget::BattleEntity,
            DiaRow::BattleMap => self.edit.type_target = TypeTarget::BattleMap,
            DiaRow::Node | DiaRow::Choice(_) => {}
        }
    }

    fn destinations(&self) -> Vec<String> {
        let mut ids: Vec<String> = self
            .project
            .talk
            .nodes
            .iter()
            .map(|node| node.id.clone())
            .collect();
        ids.push("end".into());
        ids
    }

    fn type_edit(&mut self, input: &Input) {
        if input.pause {
            self.edit.type_target = TypeTarget::None;
            return;
        }
        if let Some(ch) = input.text {
            if ch.is_ascii_graphic() && ch != ' ' || (ch == ' ' && self.allows_space()) {
                self.push_type_char(ch);
            }
        }
        if input.backspace {
            self.pop_type_char();
        }
        if input.confirm {
            self.edit.type_target = TypeTarget::None;
        }
    }

    fn allows_space(&self) -> bool {
        matches!(self.edit.type_target, TypeTarget::Weapon)
    }

    fn push_type_char(&mut self, ch: char) {
        let target = self.edit.type_target;
        let node = self.edit.node_index;
        let char_index = self.edit.char_index;
        match target {
            TypeTarget::Weapon => {
                let index = char_index.min(self.project.party.len().saturating_sub(1));
                if self.project.party[index].weapon_name.chars().count() < 24 {
                    self.project.party[index].weapon_name.push(ch);
                    self.project.dirty = true;
                    self.sync_party();
                }
            }
            TypeTarget::Require(flag) => {
                if let Some(need) = self.project.talk.nodes[node].require.get_mut(flag) {
                    if need.flag.chars().count() < 24 && ch != ' ' {
                        need.flag.push(ch);
                        self.project.dirty = true;
                    }
                }
            }
            TypeTarget::SetFlag(choice, flag) => {
                if let Some(choice) = self.project.talk.nodes[node].choices.get_mut(choice) {
                    if let Some(name) = choice.sets.get_mut(flag) {
                        if name.chars().count() < 24 && ch != ' ' {
                            name.push(ch);
                            self.project.dirty = true;
                        }
                    }
                }
            }
            TypeTarget::BattleEntity => {
                if let Some(battle) = self.project.talk.nodes[node].battle.as_mut() {
                    if battle.entity.chars().count() < 24 {
                        battle.entity.push(ch);
                        self.project.dirty = true;
                    }
                }
            }
            TypeTarget::BattleMap => {
                if let Some(battle) = self.project.talk.nodes[node].battle.as_mut() {
                    if battle.map.chars().count() < 24 {
                        battle.map.push(ch);
                        self.project.dirty = true;
                    }
                }
            }
            TypeTarget::None => {}
        }
    }

    fn pop_type_char(&mut self) {
        let target = self.edit.type_target;
        let node = self.edit.node_index;
        let char_index = self.edit.char_index;
        match target {
            TypeTarget::Weapon => {
                let index = char_index.min(self.project.party.len().saturating_sub(1));
                self.project.party[index].weapon_name.pop();
                self.project.dirty = true;
                self.sync_party();
            }
            TypeTarget::Require(flag) => {
                if let Some(need) = self.project.talk.nodes[node].require.get_mut(flag) {
                    need.flag.pop();
                    self.project.dirty = true;
                }
            }
            TypeTarget::SetFlag(choice, flag) => {
                if let Some(choice) = self.project.talk.nodes[node].choices.get_mut(choice) {
                    if let Some(name) = choice.sets.get_mut(flag) {
                        name.pop();
                        self.project.dirty = true;
                    }
                }
            }
            TypeTarget::BattleEntity => {
                if let Some(battle) = self.project.talk.nodes[node].battle.as_mut() {
                    battle.entity.pop();
                    self.project.dirty = true;
                }
            }
            TypeTarget::BattleMap => {
                if let Some(battle) = self.project.talk.nodes[node].battle.as_mut() {
                    battle.map.pop();
                    self.project.dirty = true;
                }
            }
            TypeTarget::None => {}
        }
    }

    fn step_edit_row(&mut self, tap_y: i32, len: usize) {
        if len == 0 {
            return;
        }
        if tap_y < 0 {
            self.edit.row = (self.edit.row + len - 1) % len;
        } else if tap_y > 0 {
            self.edit.row = (self.edit.row + 1) % len;
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

    fn open_talk(&mut self) {
        self.halt();
        self.talk_node = 0;
        self.talk_choice = 0;
        if self.project.talk.nodes.is_empty() {
            self.begin_fight();
            return;
        }
        if !self.requirements_met(0) {
            return;
        }
        let instant = {
            let node = &self.project.talk.nodes[0];
            node.choices.is_empty() && node.battle.is_some()
        };
        if instant {
            self.start_node_battle(0);
        } else {
            self.mode = Mode::Talk;
        }
    }

    fn update_talk(&mut self, input: &Input) {
        let choice_count = self.current_choices().len();
        if choice_count == 0 {
            if input.confirm {
                let index = self.talk_node;
                if self
                    .project
                    .talk
                    .nodes
                    .get(index)
                    .and_then(|node| node.battle.as_ref())
                    .is_some()
                {
                    self.start_node_battle(index);
                } else {
                    self.begin_fight();
                }
            }
            return;
        }
        if input.tap_y != 0 {
            if input.tap_y < 0 {
                self.talk_choice = (self.talk_choice + choice_count - 1) % choice_count;
            } else {
                self.talk_choice = (self.talk_choice + 1) % choice_count;
            }
        }
        if input.confirm {
            self.apply_choice();
        }
    }

    fn apply_choice(&mut self) {
        let node_index = self.talk_node;
        let choice_index = self.talk_choice;
        let Some(choice) = self
            .project
            .talk
            .nodes
            .get(node_index)
            .and_then(|node| node.choices.get(choice_index))
            .cloned()
        else {
            return;
        };
        for flag in choice.sets {
            self.flags.insert(flag);
        }
        if choice.goto == "end" {
            self.mode = Mode::Field;
            return;
        }
        let Some(next) = self
            .project
            .talk
            .nodes
            .iter()
            .position(|node| node.id == choice.goto)
        else {
            self.mode = Mode::Field;
            return;
        };
        self.talk_node = next;
        self.talk_choice = 0;
        if !self.requirements_met(next) {
            self.mode = Mode::Field;
            return;
        }
        let instant = {
            let node = &self.project.talk.nodes[next];
            node.choices.is_empty() && node.battle.is_some()
        };
        if instant {
            self.start_node_battle(next);
        } else {
            self.mode = Mode::Talk;
        }
    }

    fn requirements_met(&self, index: usize) -> bool {
        let Some(node) = self.project.talk.nodes.get(index) else {
            return false;
        };
        node.require
            .iter()
            .all(|need| self.flags.contains(&need.flag) == need.want)
    }

    fn start_node_battle(&mut self, index: usize) {
        let Some(link) = self
            .project
            .talk
            .nodes
            .get(index)
            .and_then(|node| node.battle.clone())
        else {
            return;
        };
        if !self.requirements_met(index) || !self.battle_link_matches(&link) {
            self.mode = Mode::Field;
            return;
        }
        self.begin_fight();
    }

    fn battle_link_matches(&self, link: &BattleLink) -> bool {
        if !self.monster_alive {
            return false;
        }
        let entity_ok = link.entity.eq_ignore_ascii_case(&self.monster.name)
            || link.entity.eq_ignore_ascii_case("slime");
        let map_ok = link.map == self.project.map_name || link.map == "field";
        entity_ok && map_ok
    }

    fn begin_fight(&mut self) {
        self.battle = Battle::fresh(&self.monster.name);
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
            Outcome::Win => {
                self.monster_alive = false;
                self.flags.insert("slime_defeated".into());
            }
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

    fn slot_path(&self, name: &str) -> PathBuf {
        self.save_dir
            .join("slots")
            .join(format!("{}.txt", file_stem(name)))
    }

    fn write_settings(&self) -> io::Result<()> {
        fs::create_dir_all(&self.save_dir)?;
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

    pub fn list_saves(&self) -> Vec<String> {
        self.list_heads()
            .into_iter()
            .map(|head| head.name)
            .collect()
    }

    fn list_heads(&self) -> Vec<SlotHead> {
        let mut heads = Vec::new();
        let Ok(entries) = fs::read_dir(self.save_dir.join("slots")) else {
            return heads;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("txt") {
                continue;
            }
            let Ok(text) = fs::read_to_string(&path) else {
                continue;
            };
            if let Some((name, stamp)) = peek_slot(&text) {
                heads.push(SlotHead { name, stamp, path });
            }
        }
        heads.sort_by(|a, b| b.stamp.cmp(&a.stamp).then(a.name.cmp(&b.name)));
        heads
    }

    #[cfg(test)]
    pub fn save_named(&mut self, name: &str) -> io::Result<String> {
        let name = name.trim();
        if name.is_empty() {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "name"));
        }
        self.write_slot(name)
    }

    pub fn overwrite_current(&mut self) -> io::Result<String> {
        let name = self.slot_name.clone().unwrap_or_else(|| "Save".into());
        self.write_slot(&name)
    }

    pub fn fork_named(&mut self, name: &str) -> io::Result<String> {
        let name = self.unique_name(name.trim());
        if name.is_empty() {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "name"));
        }
        self.write_slot(&name)
    }

    fn unique_name(&self, name: &str) -> String {
        let heads = self.list_heads();
        let mut candidate = if name.is_empty() {
            "Fork".to_string()
        } else {
            name.to_string()
        };
        for n in 0..100 {
            if n > 0 {
                candidate = format!("{name} {}", n + 1);
            }
            let stem = file_stem(&candidate);
            let clash = heads
                .iter()
                .any(|head| head.name == candidate || file_stem(&head.name) == stem);
            if !clash {
                return candidate;
            }
        }
        candidate
    }

    fn write_slot(&mut self, name: &str) -> io::Result<String> {
        let stamp = self.next_stamp();
        let path = self.slot_path(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, self.save_text(name, stamp))?;
        let _ = self.write_settings();
        self.slot_name = Some(name.to_string());
        Ok(name.to_string())
    }

    fn next_stamp(&self) -> u64 {
        let max = self
            .list_heads()
            .iter()
            .map(|head| head.stamp)
            .max()
            .unwrap_or(0);
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|dur| dur.as_millis() as u64)
            .unwrap_or(0);
        time.max(max.saturating_add(1))
    }

    fn save_text(&self, name: &str, stamp: u64) -> String {
        let mut out = String::new();
        let _ = writeln!(out, "fablespace-save 2");
        let _ = writeln!(out, "name {name}");
        let _ = writeln!(out, "stamp {stamp}");
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
        let _ = writeln!(out, "flags {}", self.flags.len());
        for flag in &self.flags {
            let _ = writeln!(out, "flag {flag}");
        }
        let _ = writeln!(out, "characters {}", self.characters.len());
        for character in &self.characters {
            character.write_block(&mut out);
        }
        out
    }

    fn load_newest(&mut self) -> bool {
        let Some(name) = self.list_saves().into_iter().next() else {
            return false;
        };
        self.load_name(&name)
    }

    fn load_name(&mut self, name: &str) -> bool {
        let Some(head) = self.list_heads().into_iter().find(|head| head.name == name) else {
            return false;
        };
        let Ok(text) = fs::read_to_string(&head.path) else {
            return false;
        };
        self.apply_save(&text)
    }

    fn apply_save(&mut self, text: &str) -> bool {
        let Some(data) = parse_save(text) else {
            return false;
        };
        let (
            Some(name),
            Some(px),
            Some(py),
            Some(hp),
            Some(max_hp),
            Some(slime_alive),
            Some(slime_hp),
            Some(slime_max),
            Some(sx),
            Some(sy),
        ) = (
            data.name,
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
        let (mut px, mut py) = self.place(px, py);
        let (sx, sy) = self.place(sx, sy);
        let alive = slime_alive && slime_hp > 0;
        if alive && px == sx && py == sy {
            let bumped = self.place(px, py + 1);
            px = bumped.0;
            py = bumped.1;
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
        self.player.name = self.characters[0].name.clone();
        self.player.atk = self.characters[0].atk;
        self.player.def = self.characters[0].def;
        self.flags = data.flags;
        self.slot_name = Some(name);
        self.party_index = 0;
        self.menu_cursor = 0;
        self.notice = None;
        self.halt();
        self.battle = Battle::fresh(&self.monster.name);
        self.mode = Mode::Field;
        let _ = self.write_settings();
        true
    }

    fn place(&self, x: i32, y: i32) -> (i32, i32) {
        if self.project.walkable(x, y) {
            return (x, y);
        }
        if self
            .project
            .walkable(self.project.start_x, self.project.start_y)
        {
            return (self.project.start_x, self.project.start_y);
        }
        for y in 0..MAP_H {
            for x in 0..MAP_W {
                if self.project.walkable(x, y) {
                    return (x, y);
                }
            }
        }
        (0, 0)
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

fn is_path_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '/' | '\\' | '.' | '-' | '_' | ':' | ' ')
}

fn file_stem(name: &str) -> String {
    let mut out = String::new();
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else if matches!(ch, ' ' | '-' | '_') && !out.ends_with('-') && !out.is_empty() {
            out.push('-');
        }
    }
    let out = out.trim_matches('-').to_string();
    if out.is_empty() {
        "save".into()
    } else {
        out
    }
}

fn peek_slot(text: &str) -> Option<(String, u64)> {
    if !text.lines().any(|line| line.trim() == "fablespace-save 2") {
        return None;
    }
    let mut name = None;
    let mut stamp = None;
    for line in text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("name ") {
            if !rest.is_empty() {
                name = Some(rest.to_string());
            }
        } else if let Some(rest) = line.strip_prefix("stamp ") {
            stamp = rest.parse::<u64>().ok();
        }
    }
    Some((name?, stamp?))
}

fn parse_save(text: &str) -> Option<SaveData> {
    let lines: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();
    if lines.first().copied() != Some("fablespace-save 2") {
        return None;
    }
    let mut data = SaveData {
        name: None,
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
        flags: BTreeSet::new(),
        characters: Vec::new(),
    };
    let mut index = 1;
    while index < lines.len() {
        let line = lines[index];
        index += 1;
        if let Some(rest) = line.strip_prefix("flags ") {
            let count: usize = rest.parse().ok()?;
            for _ in 0..count {
                let flag = lines.get(index)?.strip_prefix("flag ")?;
                index += 1;
                if flag.is_empty() {
                    return None;
                }
                data.flags.insert(flag.to_string());
            }
            continue;
        }
        if let Some(rest) = line.strip_prefix("characters ") {
            let count: usize = rest.parse().ok()?;
            for _ in 0..count {
                data.characters
                    .push(crate::project::parse_character(&lines, &mut index).ok()?);
            }
            continue;
        }
        let (key, value) = line.split_once(' ')?;
        match key {
            "name" => data.name = Some(value.to_string()),
            "stamp" => {}
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

    fn expected_terrain(x: i32, y: i32) -> Terrain {
        for building in BUILDINGS {
            if x >= building.x
                && x < building.x + building.w
                && y >= building.y
                && y < building.y + building.h
            {
                let roof = y < building.y + building.roof_rows;
                return match (building.wood, roof) {
                    (true, true) => Terrain::RoofRed,
                    (true, false) => Terrain::WallWood,
                    (false, true) => Terrain::RoofBlue,
                    (false, false) => Terrain::WallStone,
                };
            }
        }
        if x >= ROAD_X && x < ROAD_X + ROAD_W && y >= ROAD_Y0 && y < ROAD_Y1 {
            return match x - ROAD_X {
                0 => Terrain::RoadLeft,
                1 => Terrain::Road,
                _ => Terrain::RoadRight,
            };
        }
        Terrain::Grass
    }

    #[test]
    fn the_default_project_is_the_old_field() {
        let project = default_project();
        assert_eq!(project.party[0].name, "Mira");
        assert_eq!(project.party[0].weapon_name, WEAPON_NAME);
        assert_eq!(project.party[0].weapon_atk, WEAPON_ATK);
        assert_eq!(project.talk.nodes[0].text, TALK_LINE);
        assert_eq!(project.mushrooms, MUSHROOMS);
        assert_eq!(project.buildings, BUILDINGS);
        for y in 0..MAP_H {
            for x in 0..MAP_W {
                assert_eq!(project.terrain(x, y), Some(expected_terrain(x, y)));
            }
        }
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
        let game = field_game();
        assert!(game.project.road_x > MAP_W / 2);
        assert_eq!(game.project.buildings.len(), 2);
        assert_eq!(game.project.buildings[0].y, game.project.buildings[1].y);
        assert!(
            game.project.buildings[1].x
                >= game.project.buildings[0].x + game.project.buildings[0].w
        );
        for building in &game.project.buildings {
            assert!(!game.ground(building.x, building.y));
            assert!(!game.ground(building.x, building.y + building.roof_rows));
        }
        assert!(!game.ground(-1, 0));
        assert!(!game.ground(MAP_W, MAP_H - 1));
        assert!(game.ground(game.project.road_x + 1, game.project.road_y0));
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
    fn edit_is_on_the_title_and_not_in_play() {
        assert!(TITLE_ITEMS.contains(&"Edit"));
        assert!(!SYSTEM_ITEMS.contains(&"Edit"));
        let mut game = Game::with_save_dir(isolated_dir());
        game.menu_cursor = EDIT_ITEM;
        game.update(&confirm());
        assert_eq!(game.mode, Mode::Edit);
        game.update(&pause());
        assert_eq!(game.mode, Mode::Title);

        let mut game = field_game();
        game.update(&pause());
        assert!(matches!(game.mode, Mode::System(_)));
        game.update(&Input {
            tab: true,
            ..Input::default()
        });
        assert!(matches!(game.mode, Mode::System(_)));
    }

    #[test]
    fn continue_without_a_save_stays_on_the_title() {
        let mut game = Game::with_save_dir(isolated_dir());
        assert_eq!(game.mode, Mode::Title);
        game.update(&confirm());
        assert_eq!(game.mode, Mode::Title);
        assert_eq!(game.notice.as_deref(), Some("No save."));
    }

    #[test]
    fn title_quit_closes_immediately() {
        let dir = isolated_dir();
        let mut game = Game::with_save_dir(&dir);
        game.menu_cursor = 4;
        game.update(&confirm());
        assert!(game.should_quit());
        assert!(game.list_saves().is_empty());
    }

    #[test]
    fn save_keeps_place_flags_party_and_settings() {
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
        game.flags.insert("slime_fight".into());
        game.characters[0].inventory.push(Item {
            name: "Herb".into(),
        });
        game.characters.push(Character {
            name: "Jun".into(),
            weapon_name: "Bow".into(),
            weapon_atk: 2,
            hp: 18,
            max_hp: 18,
            atk: 3,
            def: 1,
            inventory: vec![Item {
                name: "Rope".into(),
            }],
            skills: vec!["Focus".into()],
        });
        let first = game.save_named("Dawn").unwrap();
        game.px = 55;
        game.py = 22;
        let second = game.fork_named("Noon").unwrap();
        assert_eq!(first, "Dawn");
        assert_eq!(second, "Noon");
        assert_eq!(
            game.list_saves(),
            vec!["Noon".to_string(), "Dawn".to_string()]
        );

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
        assert!(loaded.flags.contains("slime_fight"));
        assert_eq!(loaded.characters[0].name, "Mira");
        assert_eq!(loaded.characters[0].weapon_name, WEAPON_NAME);
        assert_eq!(loaded.characters[0].inventory[0].name, "Herb");
        assert!(loaded.characters[0].skills.is_empty());
        assert_eq!(loaded.characters[1].inventory[0].name, "Rope");
        assert_eq!(loaded.characters[1].skills[0], "Focus");

        game.px = 60;
        game.overwrite_current().unwrap();
        assert_eq!(game.list_saves()[0], "Noon");
        assert_eq!(game.list_saves().len(), 2);

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
        assert_eq!(picker.slots, vec!["Noon".to_string(), "Dawn".to_string()]);
        picker.update(&down());
        picker.update(&confirm());
        assert_eq!((picker.px, picker.py), (ROAD_X + 1, 20));
        assert_eq!(picker.slot_name.as_deref(), Some("Dawn"));

        let mut fresh = Game::with_save_dir(&dir);
        fresh.menu_cursor = 2;
        fresh.update(&confirm());
        assert_eq!(fresh.mode, Mode::Field);
        assert_eq!((fresh.px, fresh.py), (START_X, START_Y));
        assert_eq!(fresh.characters.len(), 1);
        assert!(fresh.characters[0].inventory.is_empty());
        assert!(fresh.flags.is_empty());
        assert_eq!(fresh.volume, 4);
        assert_eq!(fresh.player.hp, fresh.player.max_hp);
        assert!(fresh.monster_alive);
    }

    #[test]
    fn saving_asks_overwrite_or_fork() {
        let mut game = field_game();
        game.update(&pause());
        game.update(&confirm());
        assert!(matches!(game.mode, Mode::SaveAsk(Back::Field)));
        game.update(&confirm());
        assert_eq!(game.notice.as_deref(), Some("Saved."));
        assert_eq!(game.list_saves(), vec!["Save".to_string()]);
        game.update(&confirm());
        game.update(&down());
        game.update(&confirm());
        assert!(matches!(game.mode, Mode::NameSlot(_)));
        assert_eq!(game.name_buf, "Save fork");
        game.update(&confirm());
        assert_eq!(
            game.list_saves(),
            vec!["Save fork".to_string(), "Save".to_string()]
        );
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
        assert_eq!(game.list_saves(), vec!["Save".to_string()]);

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
    fn fight_choice_starts_the_slime_on_the_field() {
        let mut game = field_game();
        let fight = game
            .project
            .talk
            .nodes
            .iter()
            .find(|node| node.id == "fight")
            .unwrap();
        let battle = fight.battle.as_ref().unwrap();
        assert_eq!(battle.entity, "slime");
        assert_eq!(battle.map, "field");
        enter_battle(&mut game);
        assert!(game.flags.contains("slime_fight"));
        assert!(!game.flags.contains("slime_friend"));
    }

    #[test]
    fn peace_sets_the_relationship_and_does_not_fight() {
        let mut game = field_game();
        stand_next_to_slime(&mut game);
        game.update(&confirm());
        game.update(&down());
        game.update(&confirm());
        assert_eq!(game.mode, Mode::Talk);
        assert_eq!(game.current_line(), "Then stay on the path.");
        assert!(game.flags.contains("slime_friend"));
        assert!(game.monster_alive);
        game.update(&confirm());
        assert_eq!(game.mode, Mode::Field);
        assert!(game.monster_alive);
    }

    #[test]
    fn the_relationship_flag_blocks_the_fight() {
        let mut game = field_game();
        game.flags.insert("slime_friend".into());
        stand_next_to_slime(&mut game);
        game.update(&confirm());
        game.update(&confirm());
        assert_eq!(game.mode, Mode::Field);
        assert!(game.monster_alive);
        assert!(game.flags.contains("slime_fight"));
    }

    #[test]
    fn edit_paints_and_keeps_mira() {
        let mut game = field_game();
        game.mode = Mode::Edit;
        game.edit.cx = 4;
        game.edit.cy = 4;
        game.edit.brush = Brush::Void;
        game.update(&confirm());
        assert_eq!(game.project.terrain(4, 4), Some(Terrain::Void));
        assert!(!game.ground(4, 4));
        game.edit.brush = Brush::HouseWood;
        game.edit.cx = 10;
        game.edit.cy = 10;
        game.update(&confirm());
        assert_eq!(game.project.terrain(10, 10), Some(Terrain::RoofRed));
        assert_eq!(game.project.terrain(10, 13), Some(Terrain::WallWood));
        assert!(!game.ground(10, 13));

        game.edit.page = EditPage::Characters;
        game.edit.row = 1;
        game.update(&confirm());
        assert_eq!(game.project.party.len(), 2);
        assert!(!game.project.party[1]
            .inventory
            .iter()
            .any(|item| item.name == "Herb"));
        game.edit.row = 2;
        game.edit.char_index = 0;
        game.update(&confirm());
        assert_eq!(game.project.party.len(), 1);
        assert_eq!(game.project.party[0].name, "Ally 1");
        game.project.party[0].name = "Mira".into();
        game.update(&confirm());
        assert_eq!(game.notice.as_deref(), Some("Mira stays."));
        assert_eq!(game.project.party.len(), 1);
    }

    #[test]
    fn edit_sets_required_flags_and_choice_flags() {
        let mut game = field_game();
        game.mode = Mode::Edit;
        game.edit.page = EditPage::Dialogue;
        game.edit.node_index = 1;
        game.edit.row = 1;
        let before = game.project.talk.nodes[1].require[0].want;
        game.update(&Input {
            tap_x: 1,
            ..Input::default()
        });
        assert_ne!(game.project.talk.nodes[1].require[0].want, before);
        game.edit.node_index = 0;
        game.edit.row = 0;
        let add_set = game
            .dialogue_rows()
            .iter()
            .position(|row| matches!(row, DiaRow::AddSet(0)))
            .unwrap();
        game.edit.row = add_set;
        game.update(&confirm());
        assert!(game.project.talk.nodes[0].choices[0]
            .sets
            .contains(&"flag".to_string()));
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
        assert!(game.flags.contains("slime_defeated"));
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
