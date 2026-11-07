//! The default field, party, and branching talk, loaded from data files.
//!
//! The game looks beside the executable, then in `./project`, then falls
//! back to the copy compiled into the binary. Edit mode writes the same
//! files back out. Named stats, items, the skill tree, sprites, and
//! minigame slots are optional files beside those three.

use crate::content::{self, Check, Extras, StatMod};
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
#[cfg(test)]
use std::sync::OnceLock;

pub const MAP_W: i32 = 120;
pub const MAP_H: i32 = 70;
pub const TILE: i32 = 16;

const FIELD_TEXT: &str = include_str!("../project/field.txt");
const PARTY_TEXT: &str = include_str!("../project/party.txt");
const TALK_TEXT: &str = include_str!("../project/talk.txt");

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
    Void,
    Custom(u16),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Building {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub roof_rows: i32,
    pub wood: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Character {
    pub name: String,
    pub weapon_name: String,
    pub weapon_atk: i32,
    pub hp: i32,
    pub max_hp: i32,
    pub atk: i32,
    pub def: i32,
    pub inventory: Vec<Item>,
    pub skills: Vec<String>,
    /// Named stats other than hp, max hp, attack, and defense.
    pub bases: Vec<(String, i32)>,
}

impl Character {
    pub fn mira() -> Self {
        Self {
            name: "Mira".to_string(),
            weapon_name: "Short Sword".to_string(),
            weapon_atk: 3,
            hp: 20,
            max_hp: 20,
            atk: 4,
            def: 2,
            inventory: Vec::new(),
            skills: Vec::new(),
            bases: Vec::new(),
        }
    }

    pub fn ally(index: usize) -> Self {
        Self {
            name: format!("Ally {index}"),
            weapon_name: "Knife".to_string(),
            weapon_atk: 1,
            hp: 16,
            max_hp: 16,
            atk: 3,
            def: 1,
            inventory: Vec::new(),
            skills: Vec::new(),
            bases: Vec::new(),
        }
    }

    pub fn write_block(&self, out: &mut String) {
        out.push_str(&format!(
            "character {}\nweapon {}\nweapon_atk {}\nhp {}\nmax_hp {}\natk {}\ndef {}\nitems {}\n",
            self.name,
            self.weapon_name,
            self.weapon_atk,
            self.hp,
            self.max_hp,
            self.atk,
            self.def,
            self.inventory.len()
        ));
        for item in &self.inventory {
            out.push_str("item ");
            out.push_str(&item.name);
            out.push('\n');
        }
        out.push_str(&format!("skills {}\n", self.skills.len()));
        for skill in &self.skills {
            out.push_str("skill ");
            out.push_str(skill);
            out.push('\n');
        }
        for (stat, value) in &self.bases {
            out.push_str(&format!("base {stat} {value}\n"));
        }
    }
}

/// A flag the node needs before it will run. `want` false means the flag
/// must be off.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Need {
    pub flag: String,
    pub want: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Choice {
    pub label: String,
    pub goto: String,
    pub sets: Vec<String>,
    pub fail: Option<String>,
    pub checks: Vec<Check>,
    pub mods: Vec<StatMod>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BattleLink {
    pub entity: String,
    pub map: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node {
    pub id: String,
    pub text: String,
    pub require: Vec<Need>,
    pub choices: Vec<Choice>,
    pub battle: Option<BattleLink>,
    pub checks: Vec<Check>,
    pub mods: Vec<StatMod>,
    /// Named minigame slot. Combat is the slot this slice runs.
    pub slot: Option<String>,
    pub fail: Option<String>,
    pub on_win: Vec<StatMod>,
    pub on_loss: Vec<StatMod>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Talk {
    pub nodes: Vec<Node>,
}

/// Imported tiles, each 16x16 RGBA, packed one after another.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Sheet {
    pub pixels: Vec<u8>,
    pub count: i32,
}

impl Sheet {
    pub fn from_rgba(width: i32, height: i32, rgba: &[u8]) -> Result<Self, String> {
        if width < TILE || height < TILE {
            return Err("image is smaller than one tile".into());
        }
        if rgba.len() < (width * height * 4) as usize {
            return Err("image is short".into());
        }
        let cols = width / TILE;
        let rows = height / TILE;
        let mut pixels = Vec::with_capacity((cols * rows * TILE * TILE * 4) as usize);
        for ty in 0..rows {
            for tx in 0..cols {
                for y in 0..TILE {
                    for x in 0..TILE {
                        let sx = tx * TILE + x;
                        let sy = ty * TILE + y;
                        let i = ((sy * width + sx) * 4) as usize;
                        pixels.extend_from_slice(&rgba[i..i + 4]);
                    }
                }
            }
        }
        Ok(Self {
            count: cols * rows,
            pixels,
        })
    }

    pub fn tile(&self, index: i32) -> Option<&[u8]> {
        if index < 0 || index >= self.count {
            return None;
        }
        let stride = (TILE * TILE * 4) as usize;
        let start = index as usize * stride;
        self.pixels.get(start..start + stride)
    }
}

#[derive(Clone, Debug)]
pub struct Project {
    pub tiles: Vec<Terrain>,
    pub mushrooms: Vec<(i32, i32)>,
    pub buildings: Vec<Building>,
    pub road_x: i32,
    pub road_w: i32,
    pub road_y0: i32,
    pub road_y1: i32,
    pub start_x: i32,
    pub start_y: i32,
    pub map_name: String,
    pub slime_name: String,
    pub slime_x: i32,
    pub slime_y: i32,
    pub slime_hp: i32,
    pub slime_max: i32,
    pub slime_atk: i32,
    pub slime_def: i32,
    pub party: Vec<Character>,
    pub talk: Talk,
    pub sheet: Sheet,
    pub extras: Extras,
    pub dirty: bool,
}

impl Project {
    pub fn embedded() -> Self {
        Self::parse(FIELD_TEXT, PARTY_TEXT, TALK_TEXT).expect("default project")
    }

    pub fn load() -> (Self, Option<PathBuf>) {
        let mut dirs = Vec::new();
        if let Some(dir) = exe_dir() {
            dirs.push(dir.join("project"));
        }
        dirs.push(PathBuf::from("project"));
        for dir in dirs {
            if dir.join("field.txt").is_file() {
                if let Ok(project) = Self::read_dir(&dir) {
                    return (project, Some(dir));
                }
            }
        }
        let fallback = exe_dir().map(|dir| dir.join("project"));
        (Self::embedded(), fallback)
    }

    pub fn read_dir(dir: &Path) -> Result<Self, String> {
        let field = fs::read_to_string(dir.join("field.txt")).map_err(|err| err.to_string())?;
        let party = fs::read_to_string(dir.join("party.txt")).map_err(|err| err.to_string())?;
        let talk = fs::read_to_string(dir.join("talk.txt")).map_err(|err| err.to_string())?;
        let mut project = Self::parse(&field, &party, &talk)?;
        project.extras = Extras::read_dir(dir)?;
        let rgba = dir.join("tiles.rgba");
        let meta = dir.join("tiles.txt");
        if rgba.is_file() && meta.is_file() {
            if let Ok(sheet) = read_sheet(&rgba, &meta) {
                project.sheet = sheet;
            }
        }
        Ok(project)
    }

    pub fn write_dir(&self, dir: &Path) -> io::Result<()> {
        fs::create_dir_all(dir)?;
        fs::write(dir.join("field.txt"), self.field_text())?;
        fs::write(dir.join("party.txt"), self.party_text())?;
        fs::write(dir.join("talk.txt"), self.talk_text())?;
        if self.sheet.count > 0 {
            fs::write(
                dir.join("tiles.txt"),
                format!("count {}\n", self.sheet.count),
            )?;
            fs::write(dir.join("tiles.rgba"), &self.sheet.pixels)?;
        }
        self.extras.write_dir(dir)?;
        Ok(())
    }

    pub fn parse(field: &str, party: &str, talk: &str) -> Result<Self, String> {
        let mut project = parse_field(field)?;
        project.party = parse_party(party)?;
        project.talk = parse_talk(talk)?;
        if project.party.is_empty() {
            return Err("party is empty".into());
        }
        if project.talk.nodes.is_empty() {
            return Err("talk is empty".into());
        }
        Ok(project)
    }

    pub fn terrain(&self, x: i32, y: i32) -> Option<Terrain> {
        if x < 0 || y < 0 || x >= MAP_W || y >= MAP_H {
            return None;
        }
        self.tiles.get((y * MAP_W + x) as usize).copied()
    }

    pub fn set_terrain(&mut self, x: i32, y: i32, terrain: Terrain) {
        if x < 0 || y < 0 || x >= MAP_W || y >= MAP_H {
            return;
        }
        self.tiles[(y * MAP_W + x) as usize] = terrain;
        self.dirty = true;
    }

    pub fn walkable(&self, x: i32, y: i32) -> bool {
        matches!(
            self.terrain(x, y),
            Some(
                Terrain::Grass
                    | Terrain::RoadLeft
                    | Terrain::Road
                    | Terrain::RoadRight
                    | Terrain::Custom(_)
            )
        )
    }

    pub fn is_road(&self, x: i32, y: i32) -> bool {
        matches!(
            self.terrain(x, y),
            Some(Terrain::RoadLeft | Terrain::Road | Terrain::RoadRight)
        )
    }

    fn field_text(&self) -> String {
        let mut out = String::new();
        out.push_str("fablespace-field 1\n");
        out.push_str(&format!("width {MAP_W}\nheight {MAP_H}\n"));
        out.push_str(&format!("start {} {}\n", self.start_x, self.start_y));
        out.push_str(&format!(
            "slime {} {} {} {} {} {} {}\n",
            self.slime_name,
            self.slime_x,
            self.slime_y,
            self.slime_hp,
            self.slime_max,
            self.slime_atk,
            self.slime_def
        ));
        out.push_str(&format!("map {}\n", self.map_name));
        out.push_str(&format!(
            "road {} {} {} {}\n",
            self.road_x, self.road_w, self.road_y0, self.road_y1
        ));
        for (x, y) in &self.mushrooms {
            out.push_str(&format!("mushroom {x} {y}\n"));
        }
        for building in &self.buildings {
            let kind = if building.wood { "wood" } else { "stone" };
            out.push_str(&format!(
                "building {kind} {} {} {} {} {}\n",
                building.x, building.y, building.w, building.h, building.roof_rows
            ));
        }
        out.push_str("grid\n");
        for y in 0..MAP_H {
            for x in 0..MAP_W {
                out.push(terrain_char(self.tiles[(y * MAP_W + x) as usize]));
            }
            out.push('\n');
        }
        let mut custom = Vec::new();
        for y in 0..MAP_H {
            for x in 0..MAP_W {
                if let Terrain::Custom(id) = self.tiles[(y * MAP_W + x) as usize] {
                    custom.push((x, y, id));
                }
            }
        }
        if !custom.is_empty() {
            out.push_str("custom\n");
            for (x, y, id) in custom {
                out.push_str(&format!("{x} {y} {id}\n"));
            }
        }
        out
    }

    fn party_text(&self) -> String {
        let mut out = String::from("fablespace-party 1\n");
        for character in &self.party {
            character.write_block(&mut out);
        }
        out
    }

    fn talk_text(&self) -> String {
        let mut out = String::from("fablespace-talk 1\n");
        for node in &self.talk.nodes {
            out.push_str(&format!("node {}\n", node.id));
            out.push_str("text ");
            out.push_str(&node.text);
            out.push('\n');
            for need in &node.require {
                if need.want {
                    out.push_str(&format!("require {}\n", need.flag));
                } else {
                    out.push_str(&format!("require !{}\n", need.flag));
                }
            }
            for check in &node.checks {
                check.write_line(&mut out);
            }
            for modi in &node.mods {
                out.push_str(&format!("mod {} {}\n", modi.stat, modi.delta));
            }
            if let Some(slot) = &node.slot {
                out.push_str(&format!("minigame {slot}\n"));
            }
            for modi in &node.on_win {
                out.push_str(&format!("win {} {}\n", modi.stat, modi.delta));
            }
            for modi in &node.on_loss {
                out.push_str(&format!("loss {} {}\n", modi.stat, modi.delta));
            }
            if let Some(fail) = &node.fail {
                out.push_str(&format!("fail {fail}\n"));
            }
            if let Some(battle) = &node.battle {
                out.push_str(&format!("battle {} {}\n", battle.entity, battle.map));
            }
            for choice in &node.choices {
                out.push_str(&format!("choice {} {}\n", choice.label, choice.goto));
                for flag in &choice.sets {
                    out.push_str(&format!("set {flag}\n"));
                }
                for check in &choice.checks {
                    check.write_line(&mut out);
                }
                for modi in &choice.mods {
                    out.push_str(&format!("mod {} {}\n", modi.stat, modi.delta));
                }
                if let Some(fail) = &choice.fail {
                    out.push_str(&format!("fail {fail}\n"));
                }
            }
        }
        out
    }
}

#[cfg(test)]
pub fn default_project() -> &'static Project {
    static PROJECT: OnceLock<Project> = OnceLock::new();
    PROJECT.get_or_init(Project::embedded)
}

pub fn terrain_char(terrain: Terrain) -> char {
    match terrain {
        Terrain::Grass => '.',
        Terrain::RoadLeft => '<',
        Terrain::Road => '=',
        Terrain::RoadRight => '>',
        Terrain::RoofRed => 'R',
        Terrain::WallWood => 'W',
        Terrain::RoofBlue => 'B',
        Terrain::WallStone => 'S',
        Terrain::Void => '#',
        Terrain::Custom(_) => '*',
    }
}

pub fn import_png(path: &Path) -> Result<Sheet, String> {
    let bytes = fs::read(path).map_err(|err| err.to_string())?;
    let (width, height, rgba) = decode_png_bytes(&bytes)?;
    Sheet::from_rgba(width, height, &rgba)
}

pub fn decode_png_bytes(bytes: &[u8]) -> Result<(i32, i32, Vec<u8>), String> {
    let mut decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info().map_err(|err| err.to_string())?;
    let mut buf = vec![0; reader.output_buffer_size()];
    let frame = reader.next_frame(&mut buf).map_err(|err| err.to_string())?;
    let width = frame.width as i32;
    let height = frame.height as i32;
    if width > 2048 || height > 2048 || width <= 0 || height <= 0 {
        return Err("image is too large".into());
    }
    let channels = match frame.color_type {
        png::ColorType::Rgba => 4,
        png::ColorType::Rgb => 3,
        png::ColorType::GrayscaleAlpha => 2,
        png::ColorType::Grayscale => 1,
        other => return Err(format!("unsupported color {other:?}")),
    };
    let src = &buf[..frame.buffer_size()];
    let mut rgba = vec![0u8; (width * height * 4) as usize];
    for i in 0..(width * height) as usize {
        let s = i * channels;
        let (r, g, b, a) = match channels {
            4 => (src[s], src[s + 1], src[s + 2], src[s + 3]),
            3 => (src[s], src[s + 1], src[s + 2], 255),
            2 => (src[s], src[s], src[s], src[s + 1]),
            _ => (src[s], src[s], src[s], 255),
        };
        let d = i * 4;
        rgba[d] = r;
        rgba[d + 1] = g;
        rgba[d + 2] = b;
        rgba[d + 3] = a;
    }
    Ok((width, height, rgba))
}

fn read_sheet(rgba: &Path, meta: &Path) -> Result<Sheet, String> {
    let text = fs::read_to_string(meta).map_err(|err| err.to_string())?;
    let mut count = None;
    for line in text.lines() {
        if let Some(rest) = line.trim().strip_prefix("count ") {
            count = rest.parse::<i32>().ok();
        }
    }
    let count = count.ok_or("tiles.txt has no count")?;
    if count < 0 {
        return Err("bad tile count".into());
    }
    let mut file = fs::File::open(rgba).map_err(|err| err.to_string())?;
    let mut pixels = Vec::new();
    file.read_to_end(&mut pixels)
        .map_err(|err| err.to_string())?;
    let stride = (TILE * TILE * 4) as usize;
    if pixels.len() != count as usize * stride {
        return Err("tile sheet size does not match".into());
    }
    Ok(Sheet { pixels, count })
}

fn exe_dir() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))
}

fn parse_field(text: &str) -> Result<Project, String> {
    let mut width = None;
    let mut height = None;
    let mut start = None;
    let mut slime = None;
    let mut map_name = None;
    let mut road = None;
    let mut mushrooms = Vec::new();
    let mut buildings = Vec::new();
    let mut grid: Option<Vec<String>> = None;
    let mut custom = Vec::new();
    let mut in_grid = false;
    let mut in_custom = false;
    for line in text.lines() {
        if !in_grid && !in_custom {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if line == "fablespace-field 1" {
                continue;
            }
            if line == "grid" {
                in_grid = true;
                grid = Some(Vec::new());
                continue;
            }
            let mut parts = line.split_whitespace();
            let key = parts.next().ok_or("empty field line")?;
            match key {
                "width" => width = Some(number(&mut parts)?),
                "height" => height = Some(number(&mut parts)?),
                "start" => start = Some((number(&mut parts)?, number(&mut parts)?)),
                "map" => {
                    map_name = Some(parts.next().ok_or("map name")?.to_string());
                }
                "road" => {
                    road = Some((
                        number(&mut parts)?,
                        number(&mut parts)?,
                        number(&mut parts)?,
                        number(&mut parts)?,
                    ));
                }
                "mushroom" => {
                    mushrooms.push((number(&mut parts)?, number(&mut parts)?));
                }
                "slime" => {
                    let name = parts.next().ok_or("slime name")?.to_string();
                    slime = Some((
                        name,
                        number(&mut parts)?,
                        number(&mut parts)?,
                        number(&mut parts)?,
                        number(&mut parts)?,
                        number(&mut parts)?,
                        number(&mut parts)?,
                    ));
                }
                "building" => {
                    let kind = parts.next().ok_or("building kind")?;
                    let wood = match kind {
                        "wood" => true,
                        "stone" => false,
                        _ => return Err("building kind".into()),
                    };
                    buildings.push(Building {
                        x: number(&mut parts)?,
                        y: number(&mut parts)?,
                        w: number(&mut parts)?,
                        h: number(&mut parts)?,
                        roof_rows: number(&mut parts)?,
                        wood,
                    });
                }
                _ => return Err(format!("unknown field key {key}")),
            }
        } else if in_grid {
            if line == "custom" {
                in_grid = false;
                in_custom = true;
                continue;
            }
            grid.get_or_insert_with(Vec::new).push(line.to_string());
        } else if !line.trim().is_empty() {
            let mut parts = line.split_whitespace();
            custom.push((
                number(&mut parts)?,
                number(&mut parts)?,
                number(&mut parts)?,
            ));
        }
    }
    if width != Some(MAP_W) || height != Some(MAP_H) {
        return Err("map must be 120x70".into());
    }
    let (start_x, start_y) = start.ok_or("missing start")?;
    let (slime_name, slime_x, slime_y, slime_hp, slime_max, slime_atk, slime_def) =
        slime.ok_or("missing slime")?;
    let (road_x, road_w, road_y0, road_y1) = road.ok_or("missing road")?;
    let rows = grid.ok_or("missing grid")?;
    if rows.len() != MAP_H as usize {
        return Err("grid height".into());
    }
    let mut tiles = Vec::with_capacity((MAP_W * MAP_H) as usize);
    for row in &rows {
        let chars: Vec<char> = row.chars().collect();
        if chars.len() != MAP_W as usize {
            return Err("grid width".into());
        }
        for ch in chars {
            tiles.push(char_terrain(ch)?);
        }
    }
    for (x, y, id) in custom {
        if x < 0 || y < 0 || x >= MAP_W || y >= MAP_H || id < 0 {
            return Err("custom tile".into());
        }
        tiles[(y * MAP_W + x) as usize] = Terrain::Custom(id as u16);
    }
    Ok(Project {
        tiles,
        mushrooms,
        buildings,
        road_x,
        road_w,
        road_y0,
        road_y1,
        start_x,
        start_y,
        map_name: map_name.unwrap_or_else(|| "field".into()),
        slime_name,
        slime_x,
        slime_y,
        slime_hp,
        slime_max,
        slime_atk,
        slime_def,
        party: Vec::new(),
        talk: Talk { nodes: Vec::new() },
        sheet: Sheet::default(),
        extras: Extras::default(),
        dirty: false,
    })
}

fn char_terrain(ch: char) -> Result<Terrain, String> {
    Ok(match ch {
        '.' => Terrain::Grass,
        '<' => Terrain::RoadLeft,
        '=' => Terrain::Road,
        '>' => Terrain::RoadRight,
        'R' => Terrain::RoofRed,
        'W' => Terrain::WallWood,
        'B' => Terrain::RoofBlue,
        'S' => Terrain::WallStone,
        '#' => Terrain::Void,
        '*' => Terrain::Custom(0),
        _ => return Err(format!("bad tile {ch}")),
    })
}

fn number(parts: &mut std::str::SplitWhitespace<'_>) -> Result<i32, String> {
    parts
        .next()
        .ok_or("missing number")?
        .parse()
        .map_err(|_| "bad number".into())
}

pub fn parse_party(text: &str) -> Result<Vec<Character>, String> {
    let lines: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();
    if lines.first().copied() != Some("fablespace-party 1") {
        return Err("party header".into());
    }
    let mut index = 1;
    let mut party = Vec::new();
    while index < lines.len() {
        party.push(parse_character(&lines, &mut index)?);
    }
    Ok(party)
}

pub fn parse_character(lines: &[&str], index: &mut usize) -> Result<Character, String> {
    let name = lines
        .get(*index)
        .and_then(|line| line.strip_prefix("character "))
        .ok_or("character")?
        .to_string();
    *index += 1;
    if name.is_empty() {
        return Err("empty name".into());
    }
    let weapon_name = lines
        .get(*index)
        .and_then(|line| line.strip_prefix("weapon "))
        .ok_or("weapon")?
        .to_string();
    *index += 1;
    let weapon_atk = keyed(lines, index, "weapon_atk")?;
    let hp = keyed(lines, index, "hp")?;
    let max_hp = keyed(lines, index, "max_hp")?;
    let atk = keyed(lines, index, "atk")?;
    let def = keyed(lines, index, "def")?;
    let item_count: usize = keyed(lines, index, "items")?;
    let mut inventory = Vec::new();
    for _ in 0..item_count {
        let item = lines
            .get(*index)
            .and_then(|line| line.strip_prefix("item "))
            .ok_or("item")?;
        *index += 1;
        if item.is_empty() {
            return Err("empty item".into());
        }
        inventory.push(Item {
            name: item.to_string(),
        });
    }
    let skill_count: usize = keyed(lines, index, "skills")?;
    let mut skills = Vec::new();
    for _ in 0..skill_count {
        let skill = lines
            .get(*index)
            .and_then(|line| line.strip_prefix("skill "))
            .ok_or("skill")?;
        *index += 1;
        if skill.is_empty() {
            return Err("empty skill".into());
        }
        skills.push(skill.to_string());
    }
    let mut bases = Vec::new();
    while *index < lines.len() {
        let Some(rest) = lines[*index].strip_prefix("base ") else {
            break;
        };
        let mut parts = rest.split_whitespace();
        let stat = parts.next().ok_or("base")?.to_string();
        let value: i32 = parts.next().ok_or("base")?.parse().map_err(|_| "base")?;
        *index += 1;
        if stat.is_empty() {
            return Err("base".into());
        }
        bases.push((stat, value));
    }
    Ok(Character {
        name,
        weapon_name,
        weapon_atk,
        hp,
        max_hp,
        atk,
        def,
        inventory,
        skills,
        bases,
    })
}

fn keyed<T: std::str::FromStr>(lines: &[&str], index: &mut usize, key: &str) -> Result<T, String> {
    let line = lines.get(*index).ok_or_else(|| key.to_string())?;
    let prefix = format!("{key} ");
    let value = line.strip_prefix(&prefix).ok_or_else(|| key.to_string())?;
    *index += 1;
    value.parse().map_err(|_| key.to_string())
}

pub fn parse_talk(text: &str) -> Result<Talk, String> {
    let mut nodes = Vec::new();
    let mut current: Option<Node> = None;
    let mut saw_header = false;
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        if line == "fablespace-talk 1" {
            saw_header = true;
            continue;
        }
        if let Some(id) = line.strip_prefix("node ") {
            if id.is_empty() {
                return Err("empty node".into());
            }
            if let Some(node) = current.take() {
                nodes.push(node);
            }
            current = Some(Node {
                id: id.to_string(),
                text: String::new(),
                require: Vec::new(),
                choices: Vec::new(),
                battle: None,
                checks: Vec::new(),
                mods: Vec::new(),
                slot: None,
                fail: None,
                on_win: Vec::new(),
                on_loss: Vec::new(),
            });
            continue;
        }
        let node = current.as_mut().ok_or("line outside a node")?;
        if let Some(text) = line.strip_prefix("text ") {
            node.text = text.to_string();
        } else if line == "text" {
            node.text.clear();
        } else if let Some(rest) = line.strip_prefix("when ") {
            let check = content::parse_when(rest)?;
            if node.choices.is_empty() {
                node.checks.push(check);
            } else {
                node.choices
                    .last_mut()
                    .ok_or("when")?
                    .checks
                    .push(check);
            }
        } else if let Some(rest) = line.strip_prefix("mod ") {
            let modi = content::parse_mod(rest)?;
            if node.choices.is_empty() {
                node.mods.push(modi);
            } else {
                node.choices.last_mut().ok_or("mod")?.mods.push(modi);
            }
        } else if let Some(slot) = line.strip_prefix("minigame ") {
            if slot.is_empty() || slot.contains(char::is_whitespace) {
                return Err("minigame".into());
            }
            node.slot = Some(slot.to_string());
        } else if let Some(rest) = line.strip_prefix("win ") {
            node.on_win.push(content::parse_mod(rest)?);
        } else if let Some(rest) = line.strip_prefix("loss ") {
            node.on_loss.push(content::parse_mod(rest)?);
        } else if let Some(fail) = line.strip_prefix("fail ") {
            if fail.is_empty() || fail.contains(char::is_whitespace) {
                return Err("fail".into());
            }
            if node.choices.is_empty() {
                node.fail = Some(fail.to_string());
            } else {
                node.choices.last_mut().ok_or("fail")?.fail = Some(fail.to_string());
            }
        } else if let Some(flag) = line.strip_prefix("require ") {
            let (flag, want) = if let Some(flag) = flag.strip_prefix('!') {
                (flag, false)
            } else {
                (flag, true)
            };
            if flag.is_empty() {
                return Err("empty flag".into());
            }
            node.require.push(Need {
                flag: flag.to_string(),
                want,
            });
        } else if let Some(rest) = line.strip_prefix("battle ") {
            let mut parts = rest.split_whitespace();
            let entity = parts.next().ok_or("battle entity")?.to_string();
            let map = parts.next().ok_or("battle map")?.to_string();
            node.battle = Some(BattleLink { entity, map });
        } else if let Some(rest) = line.strip_prefix("choice ") {
            let mut parts: Vec<&str> = rest.split_whitespace().collect();
            if parts.len() < 2 {
                return Err("choice needs a label and a destination".into());
            }
            let goto = parts.pop().unwrap().to_string();
            let label = parts.join(" ");
            if label.is_empty() {
                return Err("empty choice".into());
            }
            node.choices.push(Choice {
                label,
                goto,
                sets: Vec::new(),
                fail: None,
                checks: Vec::new(),
                mods: Vec::new(),
            });
        } else if let Some(flag) = line.strip_prefix("set ") {
            if flag.is_empty() {
                return Err("empty set".into());
            }
            let choice = node.choices.last_mut().ok_or("set without a choice")?;
            choice.sets.push(flag.to_string());
        } else {
            return Err(format!("unknown talk line {line}"));
        }
    }
    if let Some(node) = current {
        nodes.push(node);
    }
    if !saw_header {
        return Err("talk header".into());
    }
    Ok(Talk { nodes })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shipped_files_roundtrip_the_field() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("project");
        let from_disk = Project::read_dir(&dir).unwrap();
        let embedded = Project::embedded();
        assert_eq!(from_disk.tiles, embedded.tiles);
        assert_eq!(from_disk.party, embedded.party);
        assert_eq!(from_disk.talk, embedded.talk);
        assert_eq!(from_disk.start_x, 40);
        assert_eq!(from_disk.slime_name, "Slime");
        assert_eq!(from_disk.talk.nodes[0].text, "Keep off my grass.");
        assert_eq!(from_disk.party[0].name, "Mira");
        let again = Project::parse(
            &embedded.field_text(),
            &embedded.party_text(),
            &embedded.talk_text(),
        )
        .unwrap();
        assert_eq!(again.tiles, embedded.tiles);
        assert_eq!(again.talk, embedded.talk);
        assert_eq!(again.party, embedded.party);
        assert_eq!(again.buildings, embedded.buildings);
        assert_eq!(again.mushrooms, embedded.mushrooms);
        assert_eq!(again.road_x, embedded.road_x);
        assert_eq!((again.slime_x, again.slime_y), (48, 35));
    }

    #[test]
    fn fight_node_requires_the_flag_and_names_the_battle() {
        let talk = &default_project().talk;
        let fight = talk.nodes.iter().find(|node| node.id == "fight").unwrap();
        assert!(fight
            .require
            .iter()
            .any(|need| need.flag == "slime_fight" && need.want));
        assert!(fight
            .require
            .iter()
            .any(|need| need.flag == "slime_friend" && !need.want));
        let battle = fight.battle.as_ref().unwrap();
        assert_eq!(battle.entity, "slime");
        assert_eq!(battle.map, "field");
        let greet = &talk.nodes[0];
        assert_eq!(greet.choices[0].sets, vec!["slime_fight".to_string()]);
        assert_eq!(greet.choices[1].sets, vec!["slime_friend".to_string()]);
    }

    #[test]
    fn a_png_slices_on_the_tile_grid() {
        let dir = std::env::temp_dir().join(format!("fablespace-png-{}", std::process::id()));
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("sheet.png");
        let file = fs::File::create(&path).unwrap();
        let mut encoder = png::Encoder::new(file, 32, 16);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut data = vec![0u8; 32 * 16 * 4];
        for px in data.chunks_mut(4) {
            px[0] = 10;
            px[1] = 20;
            px[2] = 30;
            px[3] = 255;
        }
        {
            let mut writer = encoder.write_header().unwrap();
            writer.write_image_data(&data).unwrap();
        }
        let sheet = import_png(&path).unwrap();
        assert_eq!(sheet.count, 2);
        assert_eq!(sheet.tile(0).unwrap()[0], 10);
        let _ = fs::remove_dir_all(&dir);
    }
}
