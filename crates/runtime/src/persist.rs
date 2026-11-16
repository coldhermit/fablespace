//! Four save slots beside the project (or beside the built executable).

use fablespace_data::Direction;
use serde::{Deserialize, Serialize};

use crate::party::{Bag, Member};
use crate::session::{Picture, Screen, Session};

pub const SAVE_VERSION: u32 = 1;
pub const SLOT_COUNT: usize = 4;

#[derive(Clone, Debug)]
pub struct SavePreview {
    pub map_name: String,
    pub play_frames: u64,
    pub gold: i32,
}

#[derive(Serialize, Deserialize)]
struct SaveFile {
    version: u32,
    map_id: u32,
    x: i32,
    y: i32,
    dir: Direction,
    party: Vec<Member>,
    gold: i32,
    items: Vec<(u32, u32)>,
    weapons: Vec<(u32, u32)>,
    armors: Vec<(u32, u32)>,
    switches: Vec<bool>,
    variables: Vec<i32>,
    self_switches: Vec<SavedSwitch>,
    play_frames: u64,
    pictures: Vec<SavedPicture>,
    event_pos: Vec<SavedEvent>,
}

#[derive(Serialize, Deserialize)]
struct SavedSwitch {
    map: u32,
    event: u32,
    key: u8,
    on: bool,
}

#[derive(Serialize, Deserialize)]
struct SavedPicture {
    id: u32,
    name: String,
    x: i32,
    y: i32,
}

#[derive(Serialize, Deserialize)]
struct SavedEvent {
    id: u32,
    x: i32,
    y: i32,
    dir: Direction,
}

impl Session {
    pub fn save_slot(&mut self, slot: usize) -> Result<(), String> {
        if slot >= SLOT_COUNT {
            return Err("that save slot does not exist".into());
        }
        self.player.moving = None;
        self.player.offset = 0;
        let file = SaveFile {
            version: SAVE_VERSION,
            map_id: self.map_id,
            x: self.player.tile_x,
            y: self.player.tile_y,
            dir: self.player.dir,
            party: self.party.clone(),
            gold: self.gold,
            items: self
                .bag
                .items
                .iter()
                .map(|(id, count)| (*id, *count))
                .collect(),
            weapons: self
                .bag
                .weapons
                .iter()
                .map(|(id, count)| (*id, *count))
                .collect(),
            armors: self
                .bag
                .armors
                .iter()
                .map(|(id, count)| (*id, *count))
                .collect(),
            switches: self.switches.clone(),
            variables: self.variables.clone(),
            self_switches: self
                .self_switches
                .iter()
                .map(|((map, event, key), on)| SavedSwitch {
                    map: *map,
                    event: *event,
                    key: *key,
                    on: *on,
                })
                .collect(),
            play_frames: self.play_frames,
            pictures: self
                .pictures
                .iter()
                .map(|picture| SavedPicture {
                    id: picture.id,
                    name: picture.name.clone(),
                    x: picture.x,
                    y: picture.y,
                })
                .collect(),
            event_pos: self
                .events
                .iter()
                .map(|event| SavedEvent {
                    id: event.id,
                    x: event.body.tile_x,
                    y: event.body.tile_y,
                    dir: event.body.dir,
                })
                .collect(),
        };
        fablespace_data::project::write_json(&self.paths.save_file(slot), &file)
    }

    pub fn load_slot(&mut self, slot: usize) -> Result<(), String> {
        if slot >= SLOT_COUNT {
            return Err("that save slot does not exist".into());
        }
        let file: SaveFile = fablespace_data::project::read_json(&self.paths.save_file(slot))?;
        if file.version != SAVE_VERSION {
            return Err("this save is from a different version".into());
        }
        self.party = file.party;
        self.gold = file.gold;
        self.bag = Bag {
            items: file.items.into_iter().collect(),
            weapons: file.weapons.into_iter().collect(),
            armors: file.armors.into_iter().collect(),
        };
        self.switches = file.switches;
        self.variables = file.variables;
        self.self_switches.clear();
        for saved in file.self_switches {
            if saved.on {
                self.self_switches
                    .insert((saved.map, saved.event, saved.key), true);
            }
        }
        self.play_frames = file.play_frames;
        self.pictures = file
            .pictures
            .into_iter()
            .map(|picture| Picture {
                id: picture.id,
                name: picture.name,
                x: picture.x,
                y: picture.y,
                from_x: picture.x,
                from_y: picture.y,
                to_x: picture.x,
                to_y: picture.y,
                frames_left: 0,
                frames_total: 0,
            })
            .collect();
        self.foreground = None;
        self.parallels.clear();
        self.message = None;
        self.choice = None;
        self.battle = None;
        self.menu = None;
        self.save_ui = None;
        self.notice.clear();
        self.transfer(
            file.map_id,
            file.x.max(0) as u32,
            file.y.max(0) as u32,
            Some(file.dir),
        );
        for saved in file.event_pos {
            if let Some(event) = self.events.iter_mut().find(|event| event.id == saved.id) {
                event.body.tile_x = saved.x;
                event.body.tile_y = saved.y;
                event.body.dir = saved.dir;
                event.body.moving = None;
                event.body.offset = 0;
            }
        }
        self.refresh_player_sprite();
        self.screen = Screen::Map;
        Ok(())
    }

    pub fn slot_preview(&self, slot: usize) -> Option<SavePreview> {
        if slot >= SLOT_COUNT {
            return None;
        }
        let file: SaveFile =
            fablespace_data::project::read_json(&self.paths.save_file(slot)).ok()?;
        Some(SavePreview {
            map_name: self.map_name(file.map_id),
            play_frames: file.play_frames,
            gold: file.gold,
        })
    }
}

pub fn format_play(frames: u64) -> String {
    let secs = frames / 60;
    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;
    format!("{hours}:{minutes:02}:{seconds:02}")
}
