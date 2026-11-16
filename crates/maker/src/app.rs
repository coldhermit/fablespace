//! The maker window: projects, the map, and the database.

mod chrome;
mod edit;

use std::path::{Path, PathBuf};

use fablespace_data::command::{self, Target};
use fablespace_data::tiles::{self, character_frame};
use fablespace_data::*;
use iced::widget::canvas::{self, Frame};
use iced::widget::image::FilterMethod;
use iced::widget::{
    button, column, container, pick_list, row, scrollable, text, text_input, Column, Row,
};
use iced::{
    event, mouse, Color, Element, Length, Point, Rectangle, Size, Subscription, Task, Theme,
};
use image::{Rgba, RgbaImage};

use crate::work::{self, Job, JobKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Map,
    Actors,
    Classes,
    Skills,
    Items,
    Weapons,
    Armors,
    Enemies,
    Troops,
    States,
    Tilesets,
    Commons,
    Switches,
    Variables,
    System,
}

impl Screen {
    fn label(self) -> &'static str {
        match self {
            Screen::Map => "Map",
            Screen::Actors => "Actors",
            Screen::Classes => "Classes",
            Screen::Skills => "Skills",
            Screen::Items => "Items",
            Screen::Weapons => "Weapons",
            Screen::Armors => "Armors",
            Screen::Enemies => "Enemies",
            Screen::Troops => "Troops",
            Screen::States => "States",
            Screen::Tilesets => "Tilesets",
            Screen::Commons => "Common Events",
            Screen::Switches => "Switches",
            Screen::Variables => "Variables",
            Screen::System => "System",
        }
    }

    fn listed(self) -> bool {
        !matches!(
            self,
            Screen::Map | Screen::Switches | Screen::Variables | Screen::System
        )
    }

    fn nav(self) -> Nav {
        match self {
            Screen::Map => Nav::World,
            Screen::Commons | Screen::Switches | Screen::Variables => Nav::Events,
            Screen::System => Nav::System,
            _ => Nav::Database,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Nav {
    World,
    Database,
    Events,
    System,
}

impl Nav {
    fn label(self) -> &'static str {
        match self {
            Nav::World => "World",
            Nav::Database => "Database",
            Nav::Events => "Events",
            Nav::System => "System",
        }
    }

    fn all() -> [Nav; 4] {
        [Nav::World, Nav::Database, Nav::Events, Nav::System]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tool {
    Pencil,
    Event,
}

#[derive(Clone, Copy, Debug)]
pub enum ImportTarget {
    Tileset,
    ActorChar,
    ActorFace,
    Enemy,
    Title,
    Picture,
}

#[derive(Clone)]
enum Purpose {
    Open,
    New,
    Import {
        category: String,
        target: ImportTarget,
    },
}

struct Browser {
    purpose: Purpose,
    dir: PathBuf,
    entries: Vec<DirEntry>,
    name: String,
    picked: Option<PathBuf>,
}

struct DirEntry {
    name: String,
    path: PathBuf,
    is_dir: bool,
}

struct Preview {
    handle: iced::widget::image::Handle,
    width: f32,
    height: f32,
}

pub struct Maker {
    pub(crate) root: Option<PathBuf>,
    pub(crate) game: Game,
    pub(crate) maps: Vec<Map>,
    pub(crate) screen: Screen,
    nav: Nav,
    pub(crate) sel: u32,
    pub(crate) map_index: usize,
    pub(crate) event: Option<u32>,
    pub(crate) page: usize,
    pub(crate) tool: Tool,
    pub(crate) lower: bool,
    pub(crate) zoom: u32,
    pub(crate) tile: u32,
    pub(crate) kind: usize,
    pub(crate) cmd_target: Option<Target>,
    pub(crate) num_key: String,
    pub(crate) num_buf: String,
    pub(crate) draft_w: String,
    pub(crate) draft_h: String,
    pub(crate) dirty: bool,
    preview_dirty: bool,
    preview: Option<Preview>,
    pub(crate) status: String,
    browser: Option<Browser>,
    job: Option<Job>,
}

#[derive(Debug, Clone)]
pub enum Message {
    BeginNew,
    BeginOpen,
    OpenSample,
    BrowseUp,
    BrowseEnter(usize),
    BrowseName(String),
    BrowseConfirm,
    BrowseCancel,
    Save,
    Playtest,
    Build,
    Tick,
    Show(Screen),
    OpenNav(Nav),
    Select(u32),
    Add,
    Delete,
    Clear,
    Text {
        key: String,
        value: String,
    },
    Num {
        key: String,
        value: String,
    },
    Pick {
        key: String,
        value: String,
    },
    Toggle {
        key: String,
        on: bool,
    },
    MapCell {
        x: u32,
        y: u32,
        left: bool,
    },
    Tool(Tool),
    Layer(bool),
    Zoom(u32),
    ChooseTile(u32),
    ChooseMap(usize),
    ChooseEvent(u32),
    DeleteEvent,
    AddPage,
    DeletePage,
    ChoosePage(usize),
    ResizeMap,
    NewMap,
    DeleteMap,
    DraftWidth(String),
    DraftHeight(String),
    CmdRow(usize),
    CmdKind(usize),
    CmdAdd,
    CmdRemove,
    CmdUp,
    CmdDown,
    ChoiceAdd,
    ChoiceRemove,
    Route {
        owner: String,
        op: String,
        index: usize,
    },
    Stack {
        kind: String,
        op: String,
        index: usize,
    },
    ListAdd {
        key: String,
    },
    ListDel {
        key: String,
        index: usize,
    },
    CyclePassage(u32),
    Import(String),
}

pub fn start() -> iced::Result {
    // `iced::executor::Default` is the tokio runtime because this crate
    // enables iced's `tokio` feature.
    iced::application(title, update, view)
        .window_size(Size::new(1360.0, 880.0))
        .theme(theme)
        .subscription(subscription)
        .executor::<iced::executor::Default>()
        .run_with(boot)
}

fn boot() -> (Maker, Task<Message>) {
    (
        Maker {
            root: None,
            game: Game::default(),
            maps: Vec::new(),
            screen: Screen::Map,
            nav: Nav::World,
            sel: 1,
            map_index: 0,
            event: None,
            page: 0,
            tool: Tool::Pencil,
            lower: true,
            zoom: 1,
            tile: 1,
            kind: 0,
            cmd_target: None,
            num_key: String::new(),
            num_buf: String::new(),
            draft_w: "17".into(),
            draft_h: "13".into(),
            dirty: false,
            preview_dirty: false,
            preview: None,
            status: "Open a project, or start a new meadow.".into(),
            browser: None,
            job: None,
        },
        Task::none(),
    )
}

fn title(maker: &Maker) -> String {
    match &maker.root {
        Some(path) => {
            let name = path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("Project");
            format!("Fablespace — {name}")
        }
        None => "Fablespace".into(),
    }
}

fn theme(_maker: &Maker) -> Theme {
    Theme::custom(
        "Fablespace".into(),
        iced::theme::Palette {
            background: Color::from_rgb(0.09, 0.13, 0.11),
            text: Color::from_rgb(0.94, 0.91, 0.84),
            primary: Color::from_rgb(0.86, 0.68, 0.28),
            success: Color::from_rgb(0.36, 0.62, 0.40),
            danger: Color::from_rgb(0.72, 0.32, 0.28),
        },
    )
}

fn subscription(maker: &Maker) -> Subscription<Message> {
    if maker.job.is_some() {
        iced::time::every(std::time::Duration::from_millis(200)).map(|_| Message::Tick)
    } else {
        Subscription::none()
    }
}

fn update(maker: &mut Maker, message: Message) -> Task<Message> {
    maker.handle(message);
    if maker.preview_dirty {
        maker.rebuild_preview();
        maker.preview_dirty = false;
    }
    Task::none()
}

fn view(maker: &Maker) -> Element<'_, Message> {
    if maker.root.is_none() && maker.browser.is_none() {
        return maker.welcome();
    }
    maker.shell()
}

impl Maker {
    fn handle(&mut self, message: Message) {
        match message {
            Message::BeginNew => self.open_browser(Purpose::New),
            Message::BeginOpen => self.open_browser(Purpose::Open),
            Message::OpenSample => {
                let path = work::workspace_root().join("sample");
                self.load_at(&path);
            }
            Message::BrowseUp => self.browse_up(),
            Message::BrowseEnter(index) => self.browse_enter(index),
            Message::BrowseName(name) => {
                if let Some(browser) = &mut self.browser {
                    browser.name = name;
                }
            }
            Message::BrowseConfirm => self.browse_confirm(),
            Message::BrowseCancel => {
                self.browser = None;
                if self.root.is_none() {
                    self.status = "Open a project, or start a new meadow.".into();
                }
            }
            Message::Save => {
                if let Err(err) = self.save_now() {
                    self.status = err;
                } else {
                    self.status = "Saved.".into();
                }
            }
            Message::Playtest => self.start_job(false),
            Message::Build => self.start_job(true),
            Message::Tick => self.poll_job(),
            Message::Show(screen) => self.focus(screen),
            Message::OpenNav(nav) => {
                self.nav = nav;
                if self.screen.nav() != nav {
                    let screen = match nav {
                        Nav::World => Screen::Map,
                        Nav::Database => Screen::Actors,
                        Nav::Events => Screen::Commons,
                        Nav::System => Screen::System,
                    };
                    self.focus(screen);
                }
            }
            Message::Select(id) => {
                self.sel = id;
                self.page = 0;
                self.cmd_target = None;
                self.num_key.clear();
            }
            Message::Add => self.add_entry(),
            Message::Delete => self.delete_last(),
            Message::Clear => self.clear_entry(),
            Message::Text { key, value } => self.apply_text(&key, value),
            Message::Num { key, value } => self.apply_num(&key, value),
            Message::Pick { key, value } => self.apply_pick(&key, value),
            Message::Toggle { key, on } => self.apply_toggle(&key, on),
            Message::MapCell { x, y, left } => self.map_cell(x, y, left),
            Message::Tool(tool) => self.tool = tool,
            Message::Layer(lower) => self.lower = lower,
            Message::Zoom(zoom) => {
                self.zoom = zoom.clamp(1, 2);
                self.preview_dirty = true;
            }
            Message::ChooseTile(id) => self.tile = id,
            Message::ChooseMap(index) => self.choose_map(index),
            Message::ChooseEvent(id) => {
                self.event = Some(id);
                self.page = 0;
                self.cmd_target = None;
                self.num_key.clear();
                self.preview_dirty = true;
            }
            Message::DeleteEvent => self.delete_event(),
            Message::AddPage => self.add_page(),
            Message::DeletePage => self.delete_page(),
            Message::ChoosePage(page) => {
                self.page = page;
                self.cmd_target = None;
                self.num_key.clear();
            }
            Message::ResizeMap => self.resize_map(),
            Message::NewMap => self.new_map(),
            Message::DeleteMap => self.delete_map(),
            Message::DraftWidth(value) => self.draft_w = value,
            Message::DraftHeight(value) => self.draft_h = value,
            Message::CmdRow(index) => self.select_command_row(index),
            Message::CmdKind(index) => self.kind = index,
            Message::CmdAdd => self.command_add(),
            Message::CmdRemove => self.command_remove(),
            Message::CmdUp => self.command_move(-1),
            Message::CmdDown => self.command_move(1),
            Message::ChoiceAdd => self.choice_add(),
            Message::ChoiceRemove => self.choice_remove(),
            Message::Route { owner, op, index } => self.apply_route(&owner, &op, index),
            Message::Stack { kind, op, index } => self.apply_stack(&kind, &op, index),
            Message::ListAdd { key } => self.list_add(&key),
            Message::ListDel { key, index } => self.list_del(&key, index),
            Message::CyclePassage(id) => self.cycle_passage(id),
            Message::Import(category) => self.begin_import(category),
        }
    }

    pub(crate) fn changed(&mut self) {
        self.dirty = true;
        self.preview_dirty = true;
    }

    pub(crate) fn shown(&self, key: &str, value: impl std::fmt::Display) -> String {
        if self.num_key == key {
            self.num_buf.clone()
        } else {
            value.to_string()
        }
    }

    fn focus(&mut self, screen: Screen) {
        self.nav = screen.nav();
        self.screen = screen;
        self.cmd_target = None;
        self.num_key.clear();
        self.page = 0;
        if screen.listed() {
            let count = self.entry_count();
            if count > 1 {
                self.sel = self.sel.clamp(1, count as u32 - 1);
            } else {
                self.sel = 0;
            }
        }
    }

    pub(crate) fn entry_count(&self) -> usize {
        match self.screen {
            Screen::Actors => self.game.actors.len(),
            Screen::Classes => self.game.classes.len(),
            Screen::Skills => self.game.skills.len(),
            Screen::Items => self.game.items.len(),
            Screen::Weapons => self.game.weapons.len(),
            Screen::Armors => self.game.armors.len(),
            Screen::Enemies => self.game.enemies.len(),
            Screen::Troops => self.game.troops.len(),
            Screen::States => self.game.states.len(),
            Screen::Tilesets => self.game.tilesets.len(),
            Screen::Commons => self.game.common_events.len(),
            _ => 0,
        }
    }

    fn add_entry(&mut self) {
        if !self.screen.listed() {
            if self.screen == Screen::Switches {
                let id = self.game.switches.len();
                self.game.switches.push(format!("Switch {id}"));
                self.changed();
            } else if self.screen == Screen::Variables {
                let id = self.game.variables.len();
                self.game.variables.push(format!("Variable {id}"));
                self.changed();
            }
            return;
        }
        let id = self.entry_count() as u32;
        match self.screen {
            Screen::Actors => self.game.actors.push(Actor::blank(id)),
            Screen::Classes => self.game.classes.push(Class::blank(id)),
            Screen::Skills => self.game.skills.push(Skill::blank(id)),
            Screen::Items => self.game.items.push(Item::blank(id)),
            Screen::Weapons => self.game.weapons.push(Weapon::blank(id)),
            Screen::Armors => self.game.armors.push(Armor::blank(id)),
            Screen::Enemies => self.game.enemies.push(Enemy::blank(id)),
            Screen::Troops => self.game.troops.push(Troop::blank(id)),
            Screen::States => self.game.states.push(State::blank(id)),
            Screen::Tilesets => self.game.tilesets.push(Tileset::blank(id)),
            Screen::Commons => self.game.common_events.push(CommonEvent::blank(id)),
            _ => return,
        }
        self.sel = id;
        self.page = 0;
        self.changed();
    }

    fn delete_last(&mut self) {
        if self.screen == Screen::Switches && self.game.switches.len() > 1 {
            self.game.switches.pop();
            self.changed();
            return;
        }
        if self.screen == Screen::Variables && self.game.variables.len() > 1 {
            self.game.variables.pop();
            self.changed();
            return;
        }
        if self.entry_count() <= 1 {
            return;
        }
        match self.screen {
            Screen::Actors => {
                self.game.actors.pop();
            }
            Screen::Classes => {
                self.game.classes.pop();
            }
            Screen::Skills => {
                self.game.skills.pop();
            }
            Screen::Items => {
                self.game.items.pop();
            }
            Screen::Weapons => {
                self.game.weapons.pop();
            }
            Screen::Armors => {
                self.game.armors.pop();
            }
            Screen::Enemies => {
                self.game.enemies.pop();
            }
            Screen::Troops => {
                self.game.troops.pop();
            }
            Screen::States => {
                self.game.states.pop();
            }
            Screen::Tilesets => {
                self.game.tilesets.pop();
            }
            Screen::Commons => {
                self.game.common_events.pop();
            }
            _ => return,
        }
        let last = self.entry_count().saturating_sub(1) as u32;
        if self.sel > last {
            self.sel = last;
        }
        self.changed();
    }

    fn clear_entry(&mut self) {
        let id = self.sel;
        if id == 0 {
            return;
        }
        match self.screen {
            Screen::Actors => {
                if let Some(slot) = self.game.actors.get_mut(id as usize) {
                    *slot = Actor::blank(id);
                }
            }
            Screen::Classes => {
                if let Some(slot) = self.game.classes.get_mut(id as usize) {
                    *slot = Class::blank(id);
                }
            }
            Screen::Skills => {
                if let Some(slot) = self.game.skills.get_mut(id as usize) {
                    *slot = Skill::blank(id);
                }
            }
            Screen::Items => {
                if let Some(slot) = self.game.items.get_mut(id as usize) {
                    *slot = Item::blank(id);
                }
            }
            Screen::Weapons => {
                if let Some(slot) = self.game.weapons.get_mut(id as usize) {
                    *slot = Weapon::blank(id);
                }
            }
            Screen::Armors => {
                if let Some(slot) = self.game.armors.get_mut(id as usize) {
                    *slot = Armor::blank(id);
                }
            }
            Screen::Enemies => {
                if let Some(slot) = self.game.enemies.get_mut(id as usize) {
                    *slot = Enemy::blank(id);
                }
            }
            Screen::Troops => {
                if let Some(slot) = self.game.troops.get_mut(id as usize) {
                    *slot = Troop::blank(id);
                }
            }
            Screen::States => {
                if let Some(slot) = self.game.states.get_mut(id as usize) {
                    *slot = State::blank(id);
                }
            }
            Screen::Tilesets => {
                if let Some(slot) = self.game.tilesets.get_mut(id as usize) {
                    *slot = Tileset::blank(id);
                }
            }
            Screen::Commons => {
                if let Some(slot) = self.game.common_events.get_mut(id as usize) {
                    *slot = CommonEvent::blank(id);
                }
            }
            _ => return,
        }
        self.changed();
    }

    pub(crate) fn map(&self) -> Option<&Map> {
        self.maps.get(self.map_index)
    }

    pub(crate) fn map_mut(&mut self) -> Option<&mut Map> {
        self.maps.get_mut(self.map_index)
    }

    pub(crate) fn event_mut(&mut self) -> Option<&mut Event> {
        let id = self.event?;
        let index = self.map_index;
        self.maps
            .get_mut(index)?
            .events
            .iter_mut()
            .find(|event| event.id == id)
    }

    pub(crate) fn page_mut(&mut self) -> Option<&mut Page> {
        let page = self.page;
        self.event_mut()?.pages.get_mut(page)
    }

    pub(crate) fn commands_mut(&mut self) -> Option<&mut Vec<Command>> {
        match self.screen {
            Screen::Map => self.page_mut().map(|page| &mut page.list),
            Screen::Commons => {
                let id = self.sel;
                self.game
                    .common_events
                    .get_mut(id as usize)
                    .filter(|_| id != 0)
                    .map(|event| &mut event.list)
            }
            Screen::Troops => {
                let id = self.sel;
                let page = self.page;
                self.game
                    .troops
                    .get_mut(id as usize)
                    .filter(|_| id != 0)?
                    .pages
                    .get_mut(page)
                    .map(|page| &mut page.list)
            }
            _ => None,
        }
    }

    pub(crate) fn command_slice(&self) -> Option<&[Command]> {
        match self.screen {
            Screen::Map => {
                let id = self.event?;
                let event = self.map()?.events.iter().find(|event| event.id == id)?;
                Some(&event.pages.get(self.page)?.list)
            }
            Screen::Commons => {
                let event = self.game.common_events.get(self.sel as usize)?;
                (self.sel != 0).then_some(&event.list)
            }
            Screen::Troops => {
                let troop = self.game.troops.get(self.sel as usize)?;
                (self.sel != 0).then_some(&troop.pages.get(self.page)?.list)
            }
            _ => None,
        }
    }

    pub(crate) fn update_cmd(&mut self, f: impl FnOnce(&mut Command) -> bool) {
        let Some(Target::Command(path)) = self.cmd_target.clone() else {
            return;
        };
        let Some(list) = self.commands_mut() else {
            return;
        };
        let Some(cmd) = command::command_mut(list, &path) else {
            return;
        };
        if f(cmd) {
            self.changed();
        }
    }

    fn choose_map(&mut self, index: usize) {
        if index >= self.maps.len() {
            return;
        }
        self.map_index = index;
        self.event = None;
        self.page = 0;
        self.cmd_target = None;
        self.num_key.clear();
        if let Some((width, height)) = self.map().map(|map| (map.width, map.height)) {
            self.draft_w = width.to_string();
            self.draft_h = height.to_string();
        }
        self.preview_dirty = true;
    }

    fn map_cell(&mut self, x: u32, y: u32, left: bool) {
        if self.tool == Tool::Event {
            let existing = self.map().and_then(|map| {
                map.events
                    .iter()
                    .find(|event| event.x == x && event.y == y)
                    .map(|event| event.id)
            });
            if let Some(id) = existing {
                self.event = Some(id);
                self.page = 0;
                self.cmd_target = None;
                self.preview_dirty = true;
                return;
            }
            if !left {
                return;
            }
            let id = self
                .map()
                .map(|map| map.events.iter().map(|event| event.id).max().unwrap_or(0) + 1)
                .unwrap_or(1);
            if let Some(map) = self.map_mut() {
                map.events.push(Event::blank(id, x, y));
                self.event = Some(id);
                self.page = 0;
                self.cmd_target = None;
                self.changed();
            }
            return;
        }
        if left {
            let tile = self.tile;
            let lower = self.lower;
            if let Some(map) = self.map_mut() {
                if lower {
                    map.set_lower(x, y, tile);
                } else {
                    map.set_upper(x, y, tile);
                }
                self.changed();
            }
            return;
        }
        let lower = self.lower;
        let id = self
            .map()
            .map(|map| {
                if lower {
                    map.lower_at(x as i32, y as i32)
                } else {
                    map.upper_at(x as i32, y as i32)
                }
            })
            .unwrap_or(0);
        self.tile = id;
        self.status = format!("Sampled tile {id}.");
    }

    fn delete_event(&mut self) {
        let Some(id) = self.event else {
            return;
        };
        if let Some(map) = self.map_mut() {
            map.events.retain(|event| event.id != id);
        }
        self.event = None;
        self.cmd_target = None;
        self.changed();
    }

    fn resize_map(&mut self) {
        let Ok(width) = self.draft_w.trim().parse::<u32>() else {
            self.status = "Width needs a whole number.".into();
            return;
        };
        let Ok(height) = self.draft_h.trim().parse::<u32>() else {
            self.status = "Height needs a whole number.".into();
            return;
        };
        let fill = self
            .map()
            .map(|map| map.lower.first().copied().unwrap_or(1))
            .unwrap_or(1);
        if let Some(map) = self.map_mut() {
            map.resize(width, height, fill);
        }
        if let Some((width, height)) = self.map().map(|map| (map.width, map.height)) {
            self.draft_w = width.to_string();
            self.draft_h = height.to_string();
        }
        if self.event_mut().is_none() {
            self.event = None;
        }
        self.changed();
        self.status = "Map size updated.".into();
    }

    fn new_map(&mut self) {
        let id = self.maps.iter().map(|map| map.id).max().unwrap_or(0) + 1;
        let tileset = self
            .game
            .tilesets
            .iter()
            .find(|tileset| tileset.id != 0)
            .map(|tileset| tileset.id)
            .unwrap_or(1);
        self.maps
            .push(Map::blank(id, format!("Map {id}"), 17, 13, tileset, 1));
        self.map_index = self.maps.len() - 1;
        self.event = None;
        self.draft_w = "17".into();
        self.draft_h = "13".into();
        self.changed();
    }

    fn delete_map(&mut self) {
        if self.maps.len() <= 1 {
            self.status = "A project keeps at least one map.".into();
            return;
        }
        let last = self.maps.len() - 1;
        let last_id = self.maps[last].id;
        let highest = self.maps.iter().map(|map| map.id).max().unwrap_or(last_id);
        if last_id != highest {
            self.status = "Delete the highest map id, so earlier transfers stay valid.".into();
            return;
        }
        self.maps.pop();
        if self.map_index >= self.maps.len() {
            self.map_index = self.maps.len() - 1;
        }
        self.event = None;
        self.changed();
    }

    fn select_command_row(&mut self, index: usize) {
        let Some(list) = self.command_slice() else {
            return;
        };
        let rows = command::rows(list);
        self.cmd_target = rows.get(index).map(|row| row.target.clone());
        self.num_key.clear();
    }

    fn save_now(&mut self) -> Result<(), String> {
        let Some(root) = self.root.clone() else {
            return Err("Open a project first.".into());
        };
        self.game.normalize();
        work::save_project(&root, &self.game, &self.maps)?;
        self.dirty = false;
        Ok(())
    }

    fn start_job(&mut self, release: bool) {
        if self.job.is_some() {
            self.status = "A build is already running.".into();
            return;
        }
        let Some(root) = self.root.clone() else {
            self.status = "Open a project first.".into();
            return;
        };
        if let Err(err) = self.save_now() {
            self.status = err;
            return;
        }
        match work::start_cargo(&root, release) {
            Ok(job) => {
                self.job = Some(job);
                self.status = if release {
                    "Building the release runtime...".into()
                } else {
                    "Building the playtest runtime...".into()
                };
            }
            Err(err) => self.status = err,
        }
    }

    fn poll_job(&mut self) {
        let Some(job) = self.job.as_mut() else {
            return;
        };
        match job.child.try_wait() {
            Ok(None) => {}
            Ok(Some(status)) if status.success() => {
                let job = self.job.take().unwrap();
                self.finish_job(job);
            }
            Ok(Some(status)) => {
                let job = self.job.take().unwrap();
                let tail = work::log_tail(&job.log_path);
                self.status = format!("Build failed ({status}).\n{tail}");
            }
            Err(err) => {
                self.job = None;
                self.status = format!("Build failed: {err}");
            }
        }
    }

    fn finish_job(&mut self, job: Job) {
        match job.kind {
            JobKind::Playtest { project } => match work::spawn_playtest(&project) {
                Ok(()) => self.status = "Playtest is running.".into(),
                Err(err) => self.status = err,
            },
            JobKind::Build { project } => match work::assemble(&project) {
                Ok(dest) => self.status = format!("Built {}", dest.display()),
                Err(err) => {
                    let tail = work::log_tail(&job.log_path);
                    self.status = if tail.is_empty() {
                        err
                    } else {
                        format!("{err}\n{tail}")
                    };
                }
            },
        }
    }

    fn load_at(&mut self, path: &Path) {
        if self.job.is_some() {
            self.status = "Wait for the build to finish.".into();
            return;
        }
        match work::load_project(path) {
            Ok((game, mut maps)) => {
                let root = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
                if maps.is_empty() {
                    let tileset = game
                        .tilesets
                        .iter()
                        .find(|t| t.id != 0)
                        .map(|t| t.id)
                        .unwrap_or(1);
                    maps.push(Map::blank(1, "Map 1", 17, 13, tileset, 1));
                    self.dirty = true;
                } else {
                    self.dirty = false;
                }
                let title = game.system.game_title.clone();
                self.game = game;
                self.maps = maps;
                self.root = Some(root);
                self.screen = Screen::Map;
                self.sel = 1;
                self.map_index = 0;
                self.event = None;
                self.page = 0;
                self.cmd_target = None;
                self.browser = None;
                self.num_key.clear();
                if let Some((width, height)) = self.map().map(|map| (map.width, map.height)) {
                    self.draft_w = width.to_string();
                    self.draft_h = height.to_string();
                }
                self.preview_dirty = true;
                self.status = format!("Opened {title}.");
            }
            Err(err) => self.status = err,
        }
    }

    fn open_browser(&mut self, purpose: Purpose) {
        if self.job.is_some() {
            self.status = "Wait for the build to finish.".into();
            return;
        }
        let dir = self.root.clone().unwrap_or_else(home_dir);
        let png = matches!(purpose, Purpose::Import { .. });
        let entries = read_entries(&dir, png);
        let name = if self.root.is_none() {
            "Fable Hollow".into()
        } else {
            self.game.system.game_title.clone()
        };
        self.browser = Some(Browser {
            purpose,
            dir,
            entries,
            name,
            picked: None,
        });
    }

    fn begin_import(&mut self, category: String) {
        let target = match category.as_str() {
            "tilesets" => ImportTarget::Tileset,
            "characters" => ImportTarget::ActorChar,
            "faces" => ImportTarget::ActorFace,
            "enemies" => ImportTarget::Enemy,
            "titles" => ImportTarget::Title,
            _ => ImportTarget::Picture,
        };
        let category = if matches!(target, ImportTarget::Picture) {
            "pictures".into()
        } else {
            category
        };
        self.open_browser(Purpose::Import { category, target });
    }

    fn browse_up(&mut self) {
        let Some(browser) = &mut self.browser else {
            return;
        };
        if let Some(parent) = browser.dir.parent() {
            browser.dir = parent.to_path_buf();
            let png = matches!(browser.purpose, Purpose::Import { .. });
            browser.entries = read_entries(&browser.dir, png);
            browser.picked = None;
        }
    }

    fn browse_enter(&mut self, index: usize) {
        let Some(browser) = &mut self.browser else {
            return;
        };
        let Some(entry) = browser.entries.get(index) else {
            return;
        };
        if entry.is_dir {
            browser.dir = entry.path.clone();
            let png = matches!(browser.purpose, Purpose::Import { .. });
            browser.entries = read_entries(&browser.dir, png);
            browser.picked = None;
        } else {
            browser.picked = Some(entry.path.clone());
        }
    }

    fn browse_confirm(&mut self) {
        let Some(browser) = self.browser.take() else {
            return;
        };
        let purpose = browser.purpose.clone();
        match purpose {
            Purpose::Open => {
                if !browser.dir.join("game.json").is_file() {
                    self.status = "Choose the folder that contains game.json.".into();
                    self.browser = Some(browser);
                    return;
                }
                self.load_at(&browser.dir);
            }
            Purpose::New => {
                let name = browser.name.trim();
                if name.is_empty() || name == "." || name == ".." || name.contains(['/', '\\']) {
                    self.status = "The project name needs to be a single folder name.".into();
                    self.browser = Some(browser);
                    return;
                }
                let dest = browser.dir.join(name);
                if dest.exists() {
                    let empty = std::fs::read_dir(&dest)
                        .map(|mut rd| rd.next().is_none())
                        .unwrap_or(false);
                    if !empty {
                        self.status = "That folder is not empty.".into();
                        self.browser = Some(browser);
                        return;
                    }
                }
                match work::write_project(&dest, Some(name)) {
                    Ok(()) => self.load_at(&dest),
                    Err(err) => {
                        self.status = err;
                        self.browser = Some(browser);
                    }
                }
            }
            Purpose::Import { category, target } => {
                let Some(root) = self.root.clone() else {
                    self.status = "Open a project before importing.".into();
                    return;
                };
                let Some(source) = browser.picked.clone() else {
                    self.status = "Choose a PNG file.".into();
                    self.browser = Some(browser);
                    return;
                };
                match work::import_png(&root, &category, &source) {
                    Ok(name) => {
                        self.apply_import(target, name.clone());
                        self.status = format!("Imported {name}.");
                    }
                    Err(err) => self.status = err,
                }
            }
        }
    }

    fn apply_import(&mut self, target: ImportTarget, name: String) {
        match target {
            ImportTarget::Tileset => {
                let id = self.sel;
                if let Some(tileset) = self.game.tilesets.get_mut(id as usize) {
                    tileset.image = name;
                }
            }
            ImportTarget::ActorChar => {
                let id = self.sel;
                if let Some(actor) = self.game.actors.get_mut(id as usize) {
                    actor.character = name;
                }
            }
            ImportTarget::ActorFace => {
                let id = self.sel;
                if let Some(actor) = self.game.actors.get_mut(id as usize) {
                    actor.face = name;
                }
            }
            ImportTarget::Enemy => {
                let id = self.sel;
                if let Some(enemy) = self.game.enemies.get_mut(id as usize) {
                    enemy.battler = name;
                }
            }
            ImportTarget::Title => self.game.system.title_image = name,
            ImportTarget::Picture => {
                self.update_cmd(|cmd| {
                    if let Command::ShowPicture { name: slot, .. } = cmd {
                        *slot = name.clone();
                        true
                    } else {
                        false
                    }
                });
                return;
            }
        }
        self.changed();
    }

    fn rebuild_preview(&mut self) {
        let Some(root) = self.root.clone() else {
            self.preview = None;
            return;
        };
        let Some(map) = self.map() else {
            self.preview = None;
            return;
        };
        let zoom = self.zoom.max(1);
        let width = map.width * TILE * zoom;
        let height = map.height * TILE * zoom;
        if width == 0 || height == 0 {
            self.preview = None;
            return;
        }
        let mut image = RgbaImage::from_pixel(width, height, Rgba([22, 32, 26, 255]));
        let tileset = self.game.tileset(map.tileset_id).cloned();
        let sheet = tileset.as_ref().and_then(|tileset| {
            load_png(&root.join(format!("img/tilesets/{}.png", tileset.image)))
        });
        if let Some(tileset) = &tileset {
            for lower in [true, false] {
                for y in 0..map.height {
                    for x in 0..map.width {
                        let Some(cell) = tiles::tile_cell(tileset, map, lower, x as i32, y as i32)
                        else {
                            continue;
                        };
                        if let Some(sheet) = &sheet {
                            stamp(
                                &mut image,
                                sheet,
                                cell.col,
                                cell.row,
                                x * TILE * zoom,
                                y * TILE * zoom,
                                zoom,
                            );
                        }
                    }
                }
            }
        }
        for y in 0..map.height {
            for x in 0..map.width {
                let x0 = x * TILE * zoom;
                let y0 = y * TILE * zoom;
                for i in 0..TILE * zoom {
                    if x0 + i < width {
                        image.put_pixel(x0 + i, y0, Rgba([12, 18, 14, 255]));
                    }
                    if y0 + i < height {
                        image.put_pixel(x0, y0 + i, Rgba([12, 18, 14, 255]));
                    }
                }
            }
        }
        let selected = self.event;
        let events: Vec<(u32, u32, u32, String, u32, Direction, u8)> = map
            .events
            .iter()
            .filter_map(|event| {
                let page = event.pages.first()?;
                Some((
                    event.id,
                    event.x,
                    event.y,
                    page.image.character.clone(),
                    page.image.index,
                    page.image.direction,
                    page.image.pattern,
                ))
            })
            .collect();
        for (id, x, y, character, index, direction, pattern) in events {
            let dx = x * TILE * zoom;
            let dy = y * TILE * zoom;
            if let Some(sheet) = load_png(&root.join(format!("img/characters/{character}.png"))) {
                let (sx, sy, sw, sh) =
                    character_frame(sheet.width(), sheet.height(), index, direction, pattern);
                stamp_rect(&mut image, &sheet, sx, sy, sw, sh, dx, dy, zoom);
            } else {
                fill_rect(
                    &mut image,
                    dx + 8 * zoom,
                    dy + 8 * zoom,
                    16 * zoom,
                    16 * zoom,
                    Rgba([230, 210, 140, 255]),
                );
            }
            if Some(id) == selected {
                frame_rect(
                    &mut image,
                    dx,
                    dy,
                    TILE * zoom,
                    TILE * zoom,
                    Rgba([230, 190, 70, 255]),
                );
            }
        }
        let raw = image.into_raw();
        self.preview = Some(Preview {
            handle: iced::widget::image::Handle::from_rgba(width, height, raw),
            width: width as f32,
            height: height as f32,
        });
    }

    fn welcome(&self) -> Element<'_, Message> {
        let sample = work::workspace_root().join("sample/game.json").is_file();
        let mut actions = column![
            chrome::action("New project", Some(Message::BeginNew)),
            text("Start from a meadow you can paint.")
                .size(14)
                .color(chrome::MUTED),
            chrome::action("Open project", Some(Message::BeginOpen)),
            text("Open a folder that contains game.json.")
                .size(14)
                .color(chrome::MUTED),
        ]
        .spacing(8);
        if sample {
            actions = actions.push(chrome::action(
                "Open Fable Hollow",
                Some(Message::OpenSample),
            ));
            actions = actions.push(
                text("The sample meadow: talk, a chest, and one battle.")
                    .size(14)
                    .color(chrome::MUTED),
            );
        }
        let card = container(chrome::card(
            "Maker",
            column![
                text("Fablespace").size(40).color(chrome::GOLD),
                text("Paint a map, write the events, then play it or build a folder you can copy.")
                    .size(18),
                actions,
                text(self.status.clone()).size(14).color(chrome::MUTED),
            ]
            .spacing(14)
            .into(),
        ))
        .width(Length::Fixed(640.0));
        container(card)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(iced::Alignment::Center)
            .align_y(iced::Alignment::Center)
            .into()
    }

    fn shell(&self) -> Element<'_, Message> {
        let subtitle = if let Some(root) = &self.root {
            let name = self.game.system.game_title.clone();
            let folder = root
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("project");
            if self.dirty {
                format!("{name}  ·  {folder}  ·  unsaved")
            } else {
                format!("{name}  ·  {folder}")
            }
        } else {
            "No project open".into()
        };
        let mut actions = row![].spacing(8);
        if self.root.is_some() {
            let busy = self.job.is_some();
            actions = actions.push(chrome::action("Save", (!busy).then_some(Message::Save)));
            actions = actions.push(chrome::action(
                "Playtest",
                (!busy).then_some(Message::Playtest),
            ));
            actions = actions.push(chrome::action("Build", (!busy).then_some(Message::Build)));
        }
        let header = chrome::bar(
            row![
                column![
                    text("Fablespace").size(20).color(chrome::GOLD),
                    text(subtitle).size(13).color(chrome::MUTED),
                ]
                .spacing(2),
                iced::widget::horizontal_space(),
                actions,
            ]
            .align_y(iced::alignment::Vertical::Center)
            .into(),
        );
        let body: Element<'_, Message> = if self.browser.is_some() {
            self.browser_view()
        } else if self.screen == Screen::Map {
            self.map_view()
        } else {
            self.database_view()
        };
        let body = if self.root.is_some() && self.browser.is_none() {
            row![self.nav(), body]
                .spacing(16)
                .height(Length::Fill)
                .into()
        } else {
            body
        };
        container(
            column![
                header,
                body,
                text(self.status.clone())
                    .size(13)
                    .color(chrome::MUTED)
                    .width(Length::Fill),
            ]
            .spacing(12)
            .padding(16),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn nav(&self) -> Element<'_, Message> {
        let mut column = Column::new().spacing(14).padding([4, 0]);
        for nav in Nav::all() {
            let open = self.nav == nav;
            let mark = if open { "▾" } else { "▸" };
            column = column.push(chrome::nav_item(
                format!("{mark}  {}", nav.label()),
                open,
                Message::OpenNav(nav),
            ));
            if !open {
                continue;
            }
            column = column.push(self.nav_pages(nav));
        }
        scrollable(column)
            .width(Length::Fixed(210.0))
            .height(Length::Fill)
            .into()
    }

    fn nav_pages(&self, nav: Nav) -> Element<'_, Message> {
        let item = |screen: Screen| {
            chrome::nav_item(
                screen.label().to_string(),
                self.screen == screen,
                Message::Show(screen),
            )
        };
        let mut column = Column::new().spacing(6).padding(iced::Padding {
            left: 8.0,
            ..iced::Padding::ZERO
        });
        match nav {
            Nav::World => column = column.push(item(Screen::Map)),
            Nav::Database => {
                column = column.push(chrome::group_label("PEOPLE"));
                column = column.push(item(Screen::Actors));
                column = column.push(item(Screen::Classes));
                column = column.push(chrome::group_label("GEAR"));
                column = column.push(item(Screen::Skills));
                column = column.push(item(Screen::Items));
                column = column.push(item(Screen::Weapons));
                column = column.push(item(Screen::Armors));
                column = column.push(chrome::group_label("FOES"));
                column = column.push(item(Screen::Enemies));
                column = column.push(item(Screen::Troops));
                column = column.push(item(Screen::States));
                column = column.push(chrome::group_label("TERRAIN"));
                column = column.push(item(Screen::Tilesets));
            }
            Nav::Events => {
                column = column.push(item(Screen::Commons));
                column = column.push(item(Screen::Switches));
                column = column.push(item(Screen::Variables));
            }
            Nav::System => column = column.push(item(Screen::System)),
        }
        column.into()
    }

    fn browser_view(&self) -> Element<'_, Message> {
        let Some(browser) = &self.browser else {
            return text("").into();
        };
        let heading = match &browser.purpose {
            Purpose::Open => "Open a project",
            Purpose::New => "New project",
            Purpose::Import { category, .. } => {
                return self.browser_body(browser, &format!("Import a PNG into img/{category}"));
            }
        };
        self.browser_body(browser, heading)
    }

    fn browser_body<'a>(&'a self, browser: &'a Browser, heading: &str) -> Element<'a, Message> {
        let mut list = Column::new().spacing(4);
        list = list.push(button(text("..")).on_press(Message::BrowseUp));
        for (index, entry) in browser.entries.iter().enumerate() {
            let mark = if browser.picked.as_ref() == Some(&entry.path) {
                "▸ "
            } else if entry.is_dir {
                ""
            } else {
                ""
            };
            let suffix = if entry.is_dir { "/" } else { "" };
            list = list.push(
                button(text(format!("{mark}{}{suffix}", entry.name)))
                    .on_press(Message::BrowseEnter(index))
                    .width(Length::Fill),
            );
        }
        let mut side = Column::new().spacing(8);
        side = side.push(text(heading.to_string()).size(22));
        side = side.push(text(browser.dir.display().to_string()));
        if matches!(browser.purpose, Purpose::New) {
            side = side.push(text("Project name"));
            side =
                side.push(text_input("Fable Hollow", &browser.name).on_input(Message::BrowseName));
        }
        let confirm = match browser.purpose {
            Purpose::Open => "Open this folder",
            Purpose::New => "Create project",
            Purpose::Import { .. } => "Import PNG",
        };
        side = side.push(
            row![
                button(text(confirm)).on_press(Message::BrowseConfirm),
                button(text("Cancel")).on_press(Message::BrowseCancel)
            ]
            .spacing(8),
        );
        row![
            scrollable(list)
                .width(Length::Fixed(360.0))
                .height(Length::Fill),
            side
        ]
        .spacing(16)
        .into()
    }

    fn map_view(&self) -> Element<'_, Message> {
        let mut maps = Column::new().spacing(6);
        for (index, map) in self.maps.iter().enumerate() {
            let label = format!("{}  {}", map.name, map.id);
            maps = maps.push(chrome::nav_item(
                label,
                index == self.map_index,
                Message::ChooseMap(index),
            ));
        }
        maps = maps.push(
            row![
                chrome::action("New map", Some(Message::NewMap)),
                chrome::action("Delete last", Some(Message::DeleteMap)),
            ]
            .spacing(8),
        );
        if let Some(map) = self.map() {
            maps = maps.push(
                text_input("Name", &map.name).on_input(|value| Message::Text {
                    key: "map.name".into(),
                    value,
                }),
            );
            let tilesets: Vec<String> = self
                .game
                .tilesets
                .iter()
                .filter(|t| t.id != 0)
                .map(|t| format!("{} {}", t.id, t.name))
                .collect();
            let current = self
                .game
                .tileset(map.tileset_id)
                .map(|t| format!("{} {}", t.id, t.name))
                .unwrap_or_default();
            let selected = tilesets
                .iter()
                .any(|item| item == &current)
                .then_some(current);
            maps = maps.push(pick_list(tilesets, selected, |value| Message::Pick {
                key: "map.tileset".into(),
                value,
            }));
        }
        maps = maps.push(
            row![
                text_input("W", &self.draft_w)
                    .on_input(Message::DraftWidth)
                    .width(Length::Fixed(64.0)),
                text_input("H", &self.draft_h)
                    .on_input(Message::DraftHeight)
                    .width(Length::Fixed(64.0)),
                chrome::action("Resize", Some(Message::ResizeMap)),
            ]
            .spacing(8)
            .align_y(iced::alignment::Vertical::Center),
        );
        let paint = column![
            text(if self.lower {
                "Lower ground"
            } else {
                "Upper decoration"
            })
            .size(14)
            .color(chrome::MUTED),
            row![
                chrome::nav_item("Lower".into(), self.lower, Message::Layer(true)),
                chrome::nav_item("Upper".into(), !self.lower, Message::Layer(false)),
            ]
            .spacing(6),
            row![
                chrome::nav_item(
                    "Pencil".into(),
                    self.tool == Tool::Pencil,
                    Message::Tool(Tool::Pencil),
                ),
                chrome::nav_item(
                    "Event".into(),
                    self.tool == Tool::Event,
                    Message::Tool(Tool::Event),
                ),
            ]
            .spacing(6),
            row![
                chrome::nav_item("1×".into(), self.zoom == 1, Message::Zoom(1)),
                chrome::nav_item("2×".into(), self.zoom == 2, Message::Zoom(2)),
            ]
            .spacing(6),
            text("Left click paints. Right click samples. Tile 0 erases. An autotile follows its four neighbors.")
                .size(13)
                .color(chrome::MUTED),
        ]
        .spacing(8);
        let mut events = Column::new().spacing(4);
        if let Some(map) = self.map() {
            for event in &map.events {
                let label = format!("{}   {}, {}", event.name, event.x, event.y);
                events = events.push(chrome::nav_item(
                    label,
                    Some(event.id) == self.event,
                    Message::ChooseEvent(event.id),
                ));
            }
            if map.events.is_empty() {
                events = events.push(
                    text("No events on this map yet.")
                        .size(13)
                        .color(chrome::MUTED),
                );
            }
        }
        let tools = column![
            chrome::card("Maps", maps.into()),
            chrome::card("Paint", paint.into()),
            chrome::card("Tiles", self.palette()),
            chrome::card("Events", events.into()),
        ]
        .spacing(12);
        let stage = column![
            chrome::card("Map", self.canvas_view()),
            self.event_section(),
        ]
        .spacing(12);
        row![
            scrollable(tools)
                .width(Length::Fixed(320.0))
                .height(Length::Fill),
            scrollable(stage).width(Length::Fill).height(Length::Fill),
        ]
        .spacing(16)
        .into()
    }

    fn canvas_view(&self) -> Element<'_, Message> {
        let Some(preview) = &self.preview else {
            return text("The map preview appears after a project is open.").into();
        };
        let tile = TILE as f32 * self.zoom as f32;
        let columns = self.map().map(|map| map.width).unwrap_or(1);
        let rows = self.map().map(|map| map.height).unwrap_or(1);
        canvas::Canvas::new(MapCanvas {
            handle: &preview.handle,
            width: preview.width,
            height: preview.height,
            tile,
            columns,
            rows,
        })
        .width(Length::Fixed(preview.width))
        .height(Length::Fixed(preview.height))
        .into()
    }

    fn palette(&self) -> Element<'_, Message> {
        let Some(map) = self.map() else {
            return text("").into();
        };
        let Some(tileset) = self.game.tileset(map.tileset_id) else {
            return text("This map has no tileset.").into();
        };
        let hidden = hidden_autotile_ids(tileset);
        let count = tile_count(self, tileset);
        let mut grid = Column::new().spacing(4);
        let mut line = Row::new().spacing(4);
        let mut placed = 0;
        for id in 0..count {
            if id != 0 && hidden.contains(&id) {
                continue;
            }
            let label = if id == 0 {
                "0".into()
            } else if tileset.autotiles.iter().any(|auto| auto.tile_id == id) {
                format!("{id}*")
            } else {
                id.to_string()
            };
            let marked = if id == self.tile {
                format!("[{label}]")
            } else {
                label
            };
            line = line.push(
                button(text(marked).size(14))
                    .on_press(Message::ChooseTile(id))
                    .width(Length::Fixed(52.0)),
            );
            placed += 1;
            if placed % 4 == 0 {
                grid = grid.push(line);
                line = Row::new().spacing(4);
            }
        }
        if placed % 4 != 0 {
            grid = grid.push(line);
        }
        grid.into()
    }
}

struct MapCanvas<'a> {
    handle: &'a iced::widget::image::Handle,
    width: f32,
    height: f32,
    tile: f32,
    columns: u32,
    rows: u32,
}

#[derive(Default)]
struct Pen {
    down: Option<bool>,
}

impl canvas::Program<Message> for MapCanvas<'_> {
    type State = Pen;

    fn update(
        &self,
        state: &mut Pen,
        event: canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> (event::Status, Option<Message>) {
        let canvas::Event::Mouse(mouse_event) = event else {
            return (event::Status::Ignored, None);
        };
        match mouse_event {
            mouse::Event::ButtonReleased(_) => {
                state.down = None;
                return (event::Status::Captured, None);
            }
            mouse::Event::ButtonPressed(button) => {
                let left = match button {
                    mouse::Button::Left => true,
                    mouse::Button::Right => false,
                    _ => return (event::Status::Ignored, None),
                };
                state.down = Some(left);
                let Some(message) = self.cell(bounds, cursor, left) else {
                    return (event::Status::Captured, None);
                };
                (event::Status::Captured, Some(message))
            }
            mouse::Event::CursorMoved { .. } => {
                let Some(left) = state.down else {
                    return (event::Status::Ignored, None);
                };
                if cursor.position_in(bounds).is_none() {
                    state.down = None;
                    return (event::Status::Ignored, None);
                }
                (event::Status::Captured, self.cell(bounds, cursor, left))
            }
            _ => (event::Status::Ignored, None),
        }
    }

    fn draw(
        &self,
        _state: &Pen,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = Frame::new(renderer, Size::new(self.width, self.height));
        frame.draw_image(
            Rectangle::new(Point::new(0.0, 0.0), Size::new(self.width, self.height)),
            canvas::Image::new(self.handle.clone())
                .filter_method(FilterMethod::Nearest)
                .snap(true),
        );
        if let Some(point) = cursor.position_in(bounds) {
            let x = (point.x / self.tile) as i32;
            let y = (point.y / self.tile) as i32;
            if x >= 0 && y >= 0 && (x as u32) < self.columns && (y as u32) < self.rows {
                frame.stroke_rectangle(
                    Point::new(x as f32 * self.tile, y as f32 * self.tile),
                    Size::new(self.tile, self.tile),
                    canvas::Stroke::default()
                        .with_width(2.0)
                        .with_color(Color::from_rgb(0.95, 0.78, 0.32)),
                );
            }
        }
        vec![frame.into_geometry()]
    }
}

impl MapCanvas<'_> {
    fn cell(&self, bounds: Rectangle, cursor: mouse::Cursor, left: bool) -> Option<Message> {
        let point = cursor.position_in(bounds)?;
        let x = (point.x / self.tile) as u32;
        let y = (point.y / self.tile) as u32;
        if x >= self.columns || y >= self.rows {
            return None;
        }
        Some(Message::MapCell { x, y, left })
    }
}

fn hidden_autotile_ids(tileset: &Tileset) -> std::collections::HashSet<u32> {
    let mut hidden = std::collections::HashSet::new();
    let cols = tileset.columns.max(1);
    for auto in &tileset.autotiles {
        for mask in 0..16u32 {
            let col = auto.origin_col + mask % 4;
            let row = auto.origin_row + mask / 4;
            let id = row * cols + col;
            if id != auto.tile_id {
                hidden.insert(id);
            }
        }
    }
    hidden
}

fn tile_count(maker: &Maker, tileset: &Tileset) -> u32 {
    let from_defs = tileset.tiles.len() as u32;
    let from_image = maker
        .root
        .as_ref()
        .and_then(|root| {
            image::image_dimensions(root.join(format!("img/tilesets/{}.png", tileset.image))).ok()
        })
        .map(|(w, h)| (w / TILE) * (h / TILE))
        .unwrap_or(0);
    from_defs.max(from_image).max(1)
}

fn home_dir() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_dir())
        .unwrap_or_else(|| PathBuf::from("."))
}

fn read_entries(dir: &Path, png: bool) -> Vec<DirEntry> {
    let mut entries = Vec::new();
    let Ok(read) = std::fs::read_dir(dir) else {
        return entries;
    };
    for entry in read.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') {
            continue;
        }
        let is_dir = path.is_dir();
        let is_png = path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("png"));
        if is_dir || (png && is_png) {
            entries.push(DirEntry { name, path, is_dir });
        }
    }
    entries.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    entries
}

fn load_png(path: &Path) -> Option<RgbaImage> {
    image::open(path).ok().map(|image| image.to_rgba8())
}

fn stamp(dst: &mut RgbaImage, sheet: &RgbaImage, col: u32, row: u32, dx: u32, dy: u32, zoom: u32) {
    stamp_rect(dst, sheet, col * TILE, row * TILE, TILE, TILE, dx, dy, zoom);
}

fn stamp_rect(
    dst: &mut RgbaImage,
    sheet: &RgbaImage,
    sx: u32,
    sy: u32,
    sw: u32,
    sh: u32,
    dx: u32,
    dy: u32,
    zoom: u32,
) {
    for y in 0..sh {
        for x in 0..sw {
            let px = sx + x;
            let py = sy + y;
            if px >= sheet.width() || py >= sheet.height() {
                continue;
            }
            let pixel = *sheet.get_pixel(px, py);
            if pixel.0[3] < 16 {
                continue;
            }
            for zy in 0..zoom {
                for zx in 0..zoom {
                    let ox = dx + x * zoom + zx;
                    let oy = dy + y * zoom + zy;
                    if ox < dst.width() && oy < dst.height() {
                        dst.put_pixel(ox, oy, pixel);
                    }
                }
            }
        }
    }
}

fn fill_rect(image: &mut RgbaImage, x: u32, y: u32, w: u32, h: u32, color: Rgba<u8>) {
    for py in y..y.saturating_add(h).min(image.height()) {
        for px in x..x.saturating_add(w).min(image.width()) {
            image.put_pixel(px, py, color);
        }
    }
}

fn frame_rect(image: &mut RgbaImage, x: u32, y: u32, w: u32, h: u32, color: Rgba<u8>) {
    if w == 0 || h == 0 {
        return;
    }
    for px in x..x.saturating_add(w).min(image.width()) {
        if y < image.height() {
            image.put_pixel(px, y, color);
        }
        let bottom = y + h - 1;
        if bottom < image.height() {
            image.put_pixel(px, bottom, color);
        }
    }
    for py in y..y.saturating_add(h).min(image.height()) {
        if x < image.width() {
            image.put_pixel(x, py, color);
        }
        let right = x + w - 1;
        if right < image.width() {
            image.put_pixel(right, py, color);
        }
    }
}
