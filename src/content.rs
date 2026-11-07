//! Named stats, items, skill tree, placements, sprites, and minigame slots.
//!
//! These files sit beside field.txt. When they are missing, the game is the
//! original field, town, Mira, and slime.

use std::fs;
use std::io::Read;
use std::path::Path;

pub const FRAME: i32 = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cmp {
    Gt,
    Ge,
    Lt,
    Le,
    Eq,
}

impl Cmp {
    pub fn eval(self, left: i32, right: i32) -> bool {
        match self {
            Cmp::Gt => left > right,
            Cmp::Ge => left >= right,
            Cmp::Lt => left < right,
            Cmp::Le => left <= right,
            Cmp::Eq => left == right,
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Some(match text {
            ">" => Cmp::Gt,
            ">=" => Cmp::Ge,
            "<" => Cmp::Lt,
            "<=" => Cmp::Le,
            "==" => Cmp::Eq,
            _ => return None,
        })
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Cmp::Gt => ">",
            Cmp::Ge => ">=",
            Cmp::Lt => "<",
            Cmp::Le => "<=",
            Cmp::Eq => "==",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatMod {
    pub stat: String,
    pub delta: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Check {
    Flag { flag: String, want: bool },
    Stat { stat: String, cmp: Cmp, value: i32 },
    Combat { win: bool },
}

impl Check {
    pub fn write_line(&self, out: &mut String) {
        match self {
            Check::Flag { flag, want } => {
                if *want {
                    out.push_str(&format!("when flag {flag}\n"));
                } else {
                    out.push_str(&format!("when flag !{flag}\n"));
                }
            }
            Check::Stat { stat, cmp, value } => {
                out.push_str(&format!("when stat {stat} {} {value}\n", cmp.as_str()));
            }
            Check::Combat { win } => {
                out.push_str(if *win {
                    "when combat win\n"
                } else {
                    "when combat loss\n"
                });
            }
        }
    }
}

pub fn parse_when(rest: &str) -> Result<Check, String> {
    let rest = rest.trim();
    if let Some(flag) = rest.strip_prefix("flag ") {
        let (flag, want) = if let Some(flag) = flag.strip_prefix('!') {
            (flag.trim(), false)
        } else {
            (flag.trim(), true)
        };
        if flag.is_empty() || flag.contains(char::is_whitespace) {
            return Err("when flag".into());
        }
        return Ok(Check::Flag {
            flag: flag.to_string(),
            want,
        });
    }
    if let Some(rest) = rest.strip_prefix("stat ") {
        let mut parts = rest.split_whitespace();
        let stat = parts.next().ok_or("when stat")?.to_string();
        let cmp = Cmp::parse(parts.next().ok_or("when cmp")?).ok_or("when cmp")?;
        let value: i32 = parts
            .next()
            .ok_or("when value")?
            .parse()
            .map_err(|_| "when value")?;
        return Ok(Check::Stat { stat, cmp, value });
    }
    match rest {
        "combat win" => Ok(Check::Combat { win: true }),
        "combat loss" => Ok(Check::Combat { win: false }),
        _ => Err("when".into()),
    }
}

pub fn parse_mod(rest: &str) -> Result<StatMod, String> {
    let mut parts = rest.split_whitespace();
    let stat = parts.next().ok_or("mod")?.to_string();
    let delta: i32 = parts.next().ok_or("mod")?.parse().map_err(|_| "mod")?;
    if stat.is_empty() || stat.contains(char::is_whitespace) {
        return Err("mod".into());
    }
    Ok(StatMod { stat, delta })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatDef {
    pub id: String,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NpcBases {
    pub name: String,
    pub bases: Vec<(String, i32)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ItemDef {
    pub id: String,
    pub name: String,
    pub folder: String,
    pub equip: Vec<StatMod>,
    pub use_mods: Vec<StatMod>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlagNeed {
    pub flag: String,
    pub want: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Placement {
    pub npc: String,
    pub map: String,
    pub x: i32,
    pub y: i32,
    pub require: Vec<FlagNeed>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatNeed {
    pub stat: String,
    pub cmp: Cmp,
    pub value: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SkillNode {
    pub id: String,
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub cost: Vec<StatMod>,
    pub need_stats: Vec<StatNeed>,
    pub need_nodes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MiniSlot {
    pub name: String,
    pub kind: String,
}

#[derive(Clone, Debug)]
pub struct SpriteStrip {
    pub owner: String,
    pub role: String,
    pub frames: i32,
    pub pixels: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MapEntity {
    pub kind: String,
    pub name: String,
    pub map: String,
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Debug, Default)]
pub struct Extras {
    pub stats: Vec<StatDef>,
    pub npc_stats: Vec<NpcBases>,
    pub catalog: Vec<ItemDef>,
    pub placements: Vec<Placement>,
    pub tree: Vec<SkillNode>,
    pub slots: Vec<MiniSlot>,
    pub sprites: Vec<SpriteStrip>,
    pub entities: Vec<MapEntity>,
}

impl Extras {
    pub fn read_dir(dir: &Path) -> Result<Self, String> {
        let mut extras = Extras::default();
        if let Some(text) = optional(dir, "stats.txt")? {
            extras.stats = parse_stats(&text, &mut extras.npc_stats)?;
        }
        if let Some(text) = optional(dir, "items.txt")? {
            extras.catalog = parse_items(&text)?;
        }
        if let Some(text) = optional(dir, "places.txt")? {
            extras.placements = parse_places(&text)?;
        }
        if let Some(text) = optional(dir, "tree.txt")? {
            extras.tree = parse_tree(&text)?;
        }
        if let Some(text) = optional(dir, "slots.txt")? {
            extras.slots = parse_slots(&text)?;
        }
        if let Some(text) = optional(dir, "entities.txt")? {
            extras.entities = parse_entities(&text)?;
        }
        if let Some(text) = optional(dir, "sprites.txt")? {
            extras.sprites = parse_sprites(dir, &text)?;
        }
        Ok(extras)
    }

    pub fn write_dir(&self, dir: &Path) -> std::io::Result<()> {
        if !self.stats.is_empty() || !self.npc_stats.is_empty() {
            fs::write(dir.join("stats.txt"), self.stats_text())?;
        }
        if !self.catalog.is_empty() {
            fs::write(dir.join("items.txt"), self.items_text())?;
        }
        if !self.placements.is_empty() {
            fs::write(dir.join("places.txt"), self.places_text())?;
        }
        if !self.tree.is_empty() {
            fs::write(dir.join("tree.txt"), self.tree_text())?;
        }
        if !self.slots.is_empty() {
            fs::write(dir.join("slots.txt"), self.slots_text())?;
        }
        if !self.entities.is_empty() {
            fs::write(dir.join("entities.txt"), self.entities_text())?;
        }
        if !self.sprites.is_empty() {
            let sprite_dir = dir.join("sprites");
            fs::create_dir_all(&sprite_dir)?;
            fs::write(dir.join("sprites.txt"), self.sprites_text())?;
            for sprite in &self.sprites {
                fs::write(sprite_dir.join(sprite_file(sprite)), &sprite.pixels)?;
            }
        }
        Ok(())
    }

    fn stats_text(&self) -> String {
        let mut out = String::from("fablespace-stats 1\n");
        for stat in &self.stats {
            out.push_str(&format!("stat {} {}\n", stat.id, stat.name));
        }
        for npc in &self.npc_stats {
            out.push_str(&format!("npc {}\n", npc.name));
            for (stat, value) in &npc.bases {
                out.push_str(&format!("base {stat} {value}\n"));
            }
        }
        out
    }

    fn items_text(&self) -> String {
        let mut out = String::from("fablespace-items 1\n");
        for item in &self.catalog {
            out.push_str(&format!("item {}\n", item.id));
            out.push_str(&format!("name {}\n", item.name));
            if !item.folder.is_empty() {
                out.push_str(&format!("folder {}\n", item.folder));
            }
            for modi in &item.equip {
                out.push_str(&format!("equip {} {}\n", modi.stat, modi.delta));
            }
            for modi in &item.use_mods {
                out.push_str(&format!("use {} {}\n", modi.stat, modi.delta));
            }
        }
        out
    }

    fn places_text(&self) -> String {
        let mut out = String::from("fablespace-places 1\n");
        for place in &self.placements {
            out.push_str(&format!(
                "place {} {} {} {}\n",
                place.npc, place.map, place.x, place.y
            ));
            for need in &place.require {
                if need.want {
                    out.push_str(&format!("require {}\n", need.flag));
                } else {
                    out.push_str(&format!("require !{}\n", need.flag));
                }
            }
        }
        out
    }

    fn tree_text(&self) -> String {
        let mut out = String::from("fablespace-tree 1\n");
        for node in &self.tree {
            out.push_str(&format!("node {}\n", node.id));
            out.push_str(&format!("name {}\n", node.name));
            out.push_str(&format!("at {} {}\n", node.x, node.y));
            for cost in &node.cost {
                out.push_str(&format!("cost {} {}\n", cost.stat, cost.delta));
            }
            for need in &node.need_stats {
                out.push_str(&format!(
                    "need stat {} {} {}\n",
                    need.stat,
                    need.cmp.as_str(),
                    need.value
                ));
            }
            for id in &node.need_nodes {
                out.push_str(&format!("need node {id}\n"));
            }
        }
        out
    }

    fn slots_text(&self) -> String {
        let mut out = String::from("fablespace-slots 1\n");
        for slot in &self.slots {
            out.push_str(&format!("slot {} {}\n", slot.name, slot.kind));
        }
        out
    }

    fn entities_text(&self) -> String {
        let mut out = String::from("fablespace-entities 1\n");
        for entity in &self.entities {
            out.push_str(&format!(
                "entity {} {} {} {} {}\n",
                entity.kind, entity.map, entity.x, entity.y, entity.name
            ));
        }
        out
    }

    fn sprites_text(&self) -> String {
        let mut out = String::from("fablespace-sprites 1\n");
        for sprite in &self.sprites {
            out.push_str(&format!(
                "sprite {} {} {}\n",
                sprite.owner, sprite.role, sprite.frames
            ));
        }
        out
    }
}

fn sprite_file(sprite: &SpriteStrip) -> String {
    format!("{}-{}.rgba", token(&sprite.owner), token(&sprite.role))
}

fn token(text: &str) -> String {
    let mut out = String::new();
    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
        }
    }
    if out.is_empty() {
        "sprite".into()
    } else {
        out
    }
}

fn optional(dir: &Path, name: &str) -> Result<Option<String>, String> {
    let path = dir.join(name);
    if !path.is_file() {
        return Ok(None);
    }
    fs::read_to_string(path).map(Some).map_err(|err| err.to_string())
}

fn parse_stats(text: &str, npcs: &mut Vec<NpcBases>) -> Result<Vec<StatDef>, String> {
    let mut stats = Vec::new();
    let mut saw = false;
    let mut current: Option<NpcBases> = None;
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        if line == "fablespace-stats 1" {
            saw = true;
            continue;
        }
        if let Some(rest) = line.strip_prefix("stat ") {
            let mut parts = rest.split_whitespace();
            let id = parts.next().ok_or("stat id")?.to_string();
            let name = parts.collect::<Vec<_>>().join(" ");
            if id.is_empty() || name.is_empty() {
                return Err("stat".into());
            }
            stats.push(StatDef { id, name });
        } else if let Some(name) = line.strip_prefix("npc ") {
            if name.is_empty() {
                return Err("npc".into());
            }
            if let Some(npc) = current.take() {
                npcs.push(npc);
            }
            current = Some(NpcBases {
                name: name.to_string(),
                bases: Vec::new(),
            });
        } else if let Some(rest) = line.strip_prefix("base ") {
            let mut parts = rest.split_whitespace();
            let stat = parts.next().ok_or("base")?.to_string();
            let value: i32 = parts.next().ok_or("base")?.parse().map_err(|_| "base")?;
            let npc = current.as_mut().ok_or("base outside npc")?;
            npc.bases.push((stat, value));
        } else {
            return Err(format!("unknown stats line {line}"));
        }
    }
    if let Some(npc) = current {
        npcs.push(npc);
    }
    if !saw {
        return Err("stats header".into());
    }
    Ok(stats)
}

fn parse_items(text: &str) -> Result<Vec<ItemDef>, String> {
    let mut items = Vec::new();
    let mut current: Option<ItemDef> = None;
    let mut saw = false;
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        if line == "fablespace-items 1" {
            saw = true;
            continue;
        }
        if let Some(id) = line.strip_prefix("item ") {
            if id.is_empty() {
                return Err("item id".into());
            }
            if let Some(item) = current.take() {
                items.push(item);
            }
            current = Some(ItemDef {
                id: id.to_string(),
                name: String::new(),
                folder: String::new(),
                equip: Vec::new(),
                use_mods: Vec::new(),
            });
            continue;
        }
        let item = current.as_mut().ok_or("line outside an item")?;
        if let Some(name) = line.strip_prefix("name ") {
            item.name = name.to_string();
        } else if let Some(folder) = line.strip_prefix("folder ") {
            item.folder = folder.to_string();
        } else if let Some(rest) = line.strip_prefix("equip ") {
            item.equip.push(parse_mod(rest)?);
        } else if let Some(rest) = line.strip_prefix("use ") {
            item.use_mods.push(parse_mod(rest)?);
        } else {
            return Err(format!("unknown item line {line}"));
        }
    }
    if let Some(item) = current {
        if item.name.is_empty() {
            return Err("item name".into());
        }
        items.push(item);
    }
    if !saw {
        return Err("items header".into());
    }
    Ok(items)
}

fn parse_places(text: &str) -> Result<Vec<Placement>, String> {
    let mut places = Vec::new();
    let mut current: Option<Placement> = None;
    let mut saw = false;
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        if line == "fablespace-places 1" {
            saw = true;
            continue;
        }
        if let Some(rest) = line.strip_prefix("place ") {
            if let Some(place) = current.take() {
                places.push(place);
            }
            let mut parts = rest.split_whitespace();
            let npc = parts.next().ok_or("place")?.to_string();
            let map = parts.next().ok_or("place")?.to_string();
            let x: i32 = parts.next().ok_or("place")?.parse().map_err(|_| "place")?;
            let y: i32 = parts.next().ok_or("place")?.parse().map_err(|_| "place")?;
            current = Some(Placement {
                npc,
                map,
                x,
                y,
                require: Vec::new(),
            });
        } else if let Some(flag) = line.strip_prefix("require ") {
            let (flag, want) = if let Some(flag) = flag.strip_prefix('!') {
                (flag, false)
            } else {
                (flag, true)
            };
            if flag.is_empty() {
                return Err("place require".into());
            }
            let place = current.as_mut().ok_or("require outside place")?;
            place.require.push(FlagNeed {
                flag: flag.to_string(),
                want,
            });
        } else {
            return Err(format!("unknown place line {line}"));
        }
    }
    if let Some(place) = current {
        places.push(place);
    }
    if !saw {
        return Err("places header".into());
    }
    Ok(places)
}

fn parse_tree(text: &str) -> Result<Vec<SkillNode>, String> {
    let mut nodes = Vec::new();
    let mut current: Option<SkillNode> = None;
    let mut saw = false;
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        if line == "fablespace-tree 1" {
            saw = true;
            continue;
        }
        if let Some(id) = line.strip_prefix("node ") {
            if id.is_empty() {
                return Err("tree node".into());
            }
            if let Some(node) = current.take() {
                nodes.push(node);
            }
            current = Some(SkillNode {
                id: id.to_string(),
                name: String::new(),
                x: 0,
                y: 0,
                cost: Vec::new(),
                need_stats: Vec::new(),
                need_nodes: Vec::new(),
            });
            continue;
        }
        let node = current.as_mut().ok_or("line outside a skill")?;
        if let Some(name) = line.strip_prefix("name ") {
            node.name = name.to_string();
        } else if let Some(rest) = line.strip_prefix("at ") {
            let mut parts = rest.split_whitespace();
            node.x = parts.next().ok_or("at")?.parse().map_err(|_| "at")?;
            node.y = parts.next().ok_or("at")?.parse().map_err(|_| "at")?;
        } else if let Some(rest) = line.strip_prefix("cost ") {
            node.cost.push(parse_mod(rest)?);
        } else if let Some(rest) = line.strip_prefix("need stat ") {
            let mut parts = rest.split_whitespace();
            let stat = parts.next().ok_or("need stat")?.to_string();
            let cmp = Cmp::parse(parts.next().ok_or("need cmp")?).ok_or("need cmp")?;
            let value: i32 = parts
                .next()
                .ok_or("need value")?
                .parse()
                .map_err(|_| "need value")?;
            node.need_stats.push(StatNeed { stat, cmp, value });
        } else if let Some(id) = line.strip_prefix("need node ") {
            if id.is_empty() {
                return Err("need node".into());
            }
            node.need_nodes.push(id.to_string());
        } else {
            return Err(format!("unknown tree line {line}"));
        }
    }
    if let Some(node) = current {
        if node.name.is_empty() {
            return Err("skill name".into());
        }
        nodes.push(node);
    }
    if !saw {
        return Err("tree header".into());
    }
    Ok(nodes)
}

fn parse_slots(text: &str) -> Result<Vec<MiniSlot>, String> {
    let mut slots = Vec::new();
    let mut saw = false;
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        if line == "fablespace-slots 1" {
            saw = true;
            continue;
        }
        let Some(rest) = line.strip_prefix("slot ") else {
            return Err(format!("unknown slot line {line}"));
        };
        let mut parts = rest.split_whitespace();
        let name = parts.next().ok_or("slot")?.to_string();
        let kind = parts.next().ok_or("slot")?.to_string();
        slots.push(MiniSlot { name, kind });
    }
    if !saw {
        return Err("slots header".into());
    }
    Ok(slots)
}

fn parse_entities(text: &str) -> Result<Vec<MapEntity>, String> {
    let mut entities = Vec::new();
    let mut saw = false;
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        if line == "fablespace-entities 1" {
            saw = true;
            continue;
        }
        let Some(rest) = line.strip_prefix("entity ") else {
            return Err(format!("unknown entity line {line}"));
        };
        let mut parts = rest.split_whitespace();
        let kind = parts.next().ok_or("entity")?.to_string();
        let map = parts.next().ok_or("entity")?.to_string();
        let x: i32 = parts.next().ok_or("entity")?.parse().map_err(|_| "entity")?;
        let y: i32 = parts.next().ok_or("entity")?.parse().map_err(|_| "entity")?;
        let name = parts.collect::<Vec<_>>().join(" ");
        if name.is_empty() {
            return Err("entity name".into());
        }
        entities.push(MapEntity { kind, name, map, x, y });
    }
    if !saw {
        return Err("entities header".into());
    }
    Ok(entities)
}

fn parse_sprites(dir: &Path, text: &str) -> Result<Vec<SpriteStrip>, String> {
    let mut sprites = Vec::new();
    let mut saw = false;
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        if line == "fablespace-sprites 1" {
            saw = true;
            continue;
        }
        let Some(rest) = line.strip_prefix("sprite ") else {
            return Err(format!("unknown sprite line {line}"));
        };
        let mut parts = rest.split_whitespace();
        let owner = parts.next().ok_or("sprite")?.to_string();
        let role = parts.next().ok_or("sprite")?.to_string();
        let frames: i32 = parts.next().ok_or("sprite")?.parse().map_err(|_| "sprite")?;
        if frames <= 0 {
            return Err("sprite frames".into());
        }
        let path = dir.join("sprites").join(sprite_file(&SpriteStrip {
            owner: owner.clone(),
            role: role.clone(),
            frames,
            pixels: Vec::new(),
        }));
        let mut file = fs::File::open(&path).map_err(|err| err.to_string())?;
        let mut pixels = Vec::new();
        file.read_to_end(&mut pixels).map_err(|err| err.to_string())?;
        let stride = (FRAME * FRAME * 4) as usize;
        if pixels.len() != frames as usize * stride {
            return Err("sprite size".into());
        }
        sprites.push(SpriteStrip {
            owner,
            role,
            frames,
            pixels,
        });
    }
    if !saw {
        return Err("sprites header".into());
    }
    Ok(sprites)
}

/// Copy authored base numbers onto a party member. Classic combat numbers
/// stay in the character fields the field already saved.
pub fn seed_bases(bases: &mut Vec<(String, i32)>, authored: &[(String, i32)]) {
    for (stat, value) in authored {
        if matches!(stat.as_str(), "hp" | "max_hp" | "atk" | "def") {
            continue;
        }
        if bases.iter().any(|(id, _)| id == stat) {
            continue;
        }
        bases.push((stat.clone(), *value));
    }
}
