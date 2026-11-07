//! Turn an author project into the files the pixel game loads.
//!
//! Save in the author stores this JSON and does not compile. Build writes
//! a portable folder: these files beside the game binary.

use crate::content::{
    seed_bases, Check, Cmp, Extras, FlagNeed, ItemDef, MapEntity, MiniSlot, NpcBases, Placement,
    SkillNode, SpriteStrip, StatDef, StatMod, StatNeed, FRAME,
};
use crate::project::{decode_png_bytes, Character, Item, Need, Node, Project, Sheet, MAP_H, MAP_W};
use base64::Engine;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorProject {
    pub title: String,
    #[serde(default)]
    pub stats: Vec<AuthorStat>,
    #[serde(default)]
    pub npcs: Vec<AuthorNpc>,
    #[serde(default)]
    pub items: Vec<AuthorItem>,
    #[serde(default)]
    pub maps: Vec<AuthorMap>,
    #[serde(default)]
    pub passages: Vec<AuthorPassage>,
    #[serde(default)]
    pub skills: Vec<AuthorSkill>,
    #[serde(default)]
    pub slots: Vec<AuthorSlot>,
    #[serde(default)]
    pub art: AuthorArt,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorStat {
    pub id: String,
    pub name: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorNpc {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub player: bool,
    #[serde(default)]
    pub weapon: String,
    #[serde(default)]
    pub weapon_atk: i32,
    #[serde(default)]
    pub bases: BTreeMap<String, i32>,
    #[serde(default)]
    pub items: Vec<String>,
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub place: Option<AuthorPlace>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorPlace {
    pub map: String,
    pub x: i32,
    pub y: i32,
    #[serde(default)]
    pub checks: Vec<AuthorCheck>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorItem {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub folder: String,
    #[serde(default)]
    pub equip: Vec<AuthorMod>,
    #[serde(default)]
    pub use_mods: Vec<AuthorMod>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AuthorMod {
    pub stat: String,
    pub delta: i32,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum AuthorCheck {
    Flag { flag: String, want: bool },
    Stat { stat: String, cmp: String, value: i32 },
    Combat { win: bool },
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorMap {
    pub id: String,
    pub name: String,
    pub width: i32,
    pub height: i32,
    pub start_x: i32,
    pub start_y: i32,
    pub grid: String,
    #[serde(default)]
    pub road_x: i32,
    #[serde(default)]
    pub road_w: i32,
    #[serde(default)]
    pub road_y0: i32,
    #[serde(default)]
    pub road_y1: i32,
    #[serde(default)]
    pub mushrooms: Vec<AuthorPoint>,
    #[serde(default)]
    pub buildings: Vec<AuthorBuilding>,
    #[serde(default)]
    pub entities: Vec<AuthorEntity>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AuthorPoint {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorBuilding {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub roof_rows: i32,
    pub wood: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AuthorEntity {
    pub kind: String,
    pub name: String,
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorPassage {
    pub id: String,
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub checks: Vec<AuthorCheck>,
    #[serde(default)]
    pub choices: Vec<AuthorChoice>,
    #[serde(default)]
    pub mods: Vec<AuthorMod>,
    #[serde(default)]
    pub slot: Option<String>,
    #[serde(default)]
    pub battle: Option<AuthorBattle>,
    #[serde(default)]
    pub fail: Option<String>,
    #[serde(default)]
    pub on_win: Vec<AuthorMod>,
    #[serde(default)]
    pub on_loss: Vec<AuthorMod>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorChoice {
    pub label: String,
    pub goto: String,
    #[serde(default)]
    pub fail: Option<String>,
    #[serde(default)]
    pub sets: Vec<String>,
    #[serde(default)]
    pub checks: Vec<AuthorCheck>,
    #[serde(default)]
    pub mods: Vec<AuthorMod>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AuthorBattle {
    pub entity: String,
    pub map: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorSkill {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub x: i32,
    #[serde(default)]
    pub y: i32,
    #[serde(default)]
    pub cost: Vec<AuthorMod>,
    #[serde(default)]
    pub need_stats: Vec<AuthorCheck>,
    #[serde(default)]
    pub need_nodes: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AuthorSlot {
    pub name: String,
    pub kind: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorArt {
    #[serde(default)]
    pub tiles: Option<String>,
    #[serde(default)]
    pub sprites: Vec<AuthorSprite>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorSprite {
    pub owner: String,
    pub role: String,
    pub frames: i32,
    pub data: String,
}

pub fn compile_project(json: &str) -> Result<Project, String> {
    let author: AuthorProject = serde_json::from_str(json).map_err(|err| err.to_string())?;
    author.into_project()
}

impl AuthorProject {
    pub fn into_project(self) -> Result<Project, String> {
        let map = self
            .maps
            .first()
            .ok_or("the project needs a map")?
            .clone();
        if map.width != MAP_W || map.height != MAP_H {
            return Err("map must be 120x70".into());
        }
        let mut base = Project::parse(&map.field_text(&self)?, &self.party_text()?, "fablespace-talk 1\nnode temp\ntext hi\nchoice Ok end\n")?;
        // party and talk are replaced below; the temp talk only satisfies parse.
        base.party = self.party()?;
        base.talk.nodes = self.nodes()?;
        base.extras = self.extras()?;
        base.sheet = self.sheet()?;
        base.dirty = false;
        if base.party.is_empty() {
            return Err("party is empty".into());
        }
        if base.talk.nodes.is_empty() {
            return Err("talk is empty".into());
        }
        Ok(base)
    }

    fn party(&self) -> Result<Vec<Character>, String> {
        let mut party = Vec::new();
        for npc in self.npcs.iter().filter(|npc| npc.player) {
            party.push(npc.character()?);
        }
        if party.is_empty() {
            if let Some(npc) = self.npcs.first() {
                party.push(npc.character()?);
            }
        }
        Ok(party)
    }

    fn party_text(&self) -> Result<String, String> {
        let party = self.party()?;
        let mut out = String::from("fablespace-party 1\n");
        for character in &party {
            character.write_block(&mut out);
        }
        Ok(out)
    }

    fn nodes(&self) -> Result<Vec<Node>, String> {
        let mut nodes = Vec::new();
        for passage in &self.passages {
            if passage.kind == "terminus" || passage.kind == "place" {
                continue;
            }
            let (require, checks) = split_checks(&passage.checks)?;
            nodes.push(Node {
                id: passage.id.clone(),
                text: passage.text.clone(),
                require,
                choices: passage
                    .choices
                    .iter()
                    .map(|choice| choice.to_choice())
                    .collect::<Result<Vec<_>, _>>()?,
                battle: passage.battle.as_ref().map(|battle| crate::project::BattleLink {
                    entity: battle.entity.clone(),
                    map: battle.map.clone(),
                }),
                checks,
                mods: mods(&passage.mods),
                slot: passage.slot.clone(),
                fail: passage.fail.clone(),
                on_win: mods(&passage.on_win),
                on_loss: mods(&passage.on_loss),
            });
        }
        Ok(nodes)
    }

    fn extras(&self) -> Result<Extras, String> {
        let mut extras = Extras {
            stats: self
                .stats
                .iter()
                .map(|stat| StatDef {
                    id: stat.id.clone(),
                    name: stat.name.clone(),
                })
                .collect(),
            npc_stats: self
                .npcs
                .iter()
                .map(|npc| NpcBases {
                    name: npc.name.clone(),
                    bases: npc.bases.iter().map(|(k, v)| (k.clone(), *v)).collect(),
                })
                .collect(),
            catalog: self
                .items
                .iter()
                .map(|item| ItemDef {
                    id: item.id.clone(),
                    name: item.name.clone(),
                    folder: item.folder.clone(),
                    equip: mods(&item.equip),
                    use_mods: mods(&item.use_mods),
                })
                .collect(),
            placements: Vec::new(),
            tree: Vec::new(),
            slots: self
                .slots
                .iter()
                .map(|slot| MiniSlot {
                    name: slot.name.clone(),
                    kind: slot.kind.clone(),
                })
                .collect(),
            sprites: Vec::new(),
            entities: Vec::new(),
        };
        for npc in &self.npcs {
            if let Some(place) = &npc.place {
                let mut require = Vec::new();
                for check in &place.checks {
                    if let AuthorCheck::Flag { flag, want } = check {
                        require.push(FlagNeed {
                            flag: flag.clone(),
                            want: *want,
                        });
                    }
                }
                extras.placements.push(Placement {
                    npc: npc.name.clone(),
                    map: place.map.clone(),
                    x: place.x,
                    y: place.y,
                    require,
                });
            }
        }
        for skill in &self.skills {
            let mut need_stats = Vec::new();
            for check in &skill.need_stats {
                if let AuthorCheck::Stat { stat, cmp, value } = check {
                    need_stats.push(StatNeed {
                        stat: stat.clone(),
                        cmp: Cmp::parse(cmp).ok_or_else(|| format!("bad compare {cmp}"))?,
                        value: *value,
                    });
                }
            }
            extras.tree.push(SkillNode {
                id: skill.id.clone(),
                name: skill.name.clone(),
                x: skill.x,
                y: skill.y,
                cost: mods(&skill.cost),
                need_stats,
                need_nodes: skill.need_nodes.clone(),
            });
        }
        if let Some(map) = self.maps.first() {
            for entity in &map.entities {
                extras.entities.push(MapEntity {
                    kind: entity.kind.clone(),
                    name: entity.name.clone(),
                    map: map.id.clone(),
                    x: entity.x,
                    y: entity.y,
                });
            }
        }
        for sprite in &self.art.sprites {
            extras.sprites.push(decode_sprite(sprite)?);
        }
        Ok(extras)
    }

    fn sheet(&self) -> Result<Sheet, String> {
        let Some(data) = &self.art.tiles else {
            return Ok(Sheet::default());
        };
        if data.is_empty() {
            return Ok(Sheet::default());
        }
        let bytes = decode_data(&data)?;
        let (width, height, rgba) = decode_png_bytes(&bytes)?;
        Sheet::from_rgba(width, height, &rgba)
    }
}

impl AuthorNpc {
    fn character(&self) -> Result<Character, String> {
        let hp = *self.bases.get("hp").ok_or("npc needs hp")?;
        let max_hp = *self.bases.get("max_hp").unwrap_or(&hp);
        let atk = *self.bases.get("atk").unwrap_or(&0);
        let def = *self.bases.get("def").unwrap_or(&0);
        let mut bases = Vec::new();
        let pairs: Vec<(String, i32)> = self.bases.iter().map(|(k, v)| (k.clone(), *v)).collect();
        seed_bases(&mut bases, &pairs);
        Ok(Character {
            name: self.name.clone(),
            weapon_name: if self.weapon.is_empty() {
                "Fist".into()
            } else {
                self.weapon.clone()
            },
            weapon_atk: self.weapon_atk,
            hp,
            max_hp,
            atk,
            def,
            inventory: self
                .items
                .iter()
                .map(|name| Item { name: name.clone() })
                .collect(),
            skills: self.skills.clone(),
            bases,
        })
    }
}

impl AuthorMap {
    fn field_text(&self, project: &AuthorProject) -> Result<String, String> {
        let rows: Vec<&str> = self.grid.split('\n').filter(|row| !row.is_empty()).collect();
        if rows.len() != MAP_H as usize {
            return Err("grid height".into());
        }
        for row in &rows {
            if row.chars().count() != MAP_W as usize {
                return Err("grid width".into());
            }
        }
        let (name, x, y, hp, max_hp, atk, def) = slime_line(project);
        let mut out = String::new();
        out.push_str("fablespace-field 1\n");
        out.push_str(&format!("width {MAP_W}\nheight {MAP_H}\n"));
        out.push_str(&format!("start {} {}\n", self.start_x, self.start_y));
        out.push_str(&format!(
            "slime {name} {x} {y} {hp} {max_hp} {atk} {def}\n"
        ));
        out.push_str(&format!("map {}\n", self.id));
        out.push_str(&format!(
            "road {} {} {} {}\n",
            self.road_x, self.road_w, self.road_y0, self.road_y1
        ));
        for point in &self.mushrooms {
            out.push_str(&format!("mushroom {} {}\n", point.x, point.y));
        }
        for building in &self.buildings {
            let kind = if building.wood { "wood" } else { "stone" };
            out.push_str(&format!(
                "building {kind} {} {} {} {} {}\n",
                building.x, building.y, building.w, building.h, building.roof_rows
            ));
        }
        out.push_str("grid\n");
        for row in rows {
            out.push_str(row);
            out.push('\n');
        }
        Ok(out)
    }
}

fn slime_line(project: &AuthorProject) -> (String, i32, i32, i32, i32, i32, i32) {
    let slime = project
        .npcs
        .iter()
        .find(|npc| npc.id == "slime" || npc.name.eq_ignore_ascii_case("slime"));
    if let Some(slime) = slime {
        let hp = slime.bases.get("hp").copied().unwrap_or(12);
        let max_hp = slime.bases.get("max_hp").copied().unwrap_or(hp);
        let atk = slime.bases.get("atk").copied().unwrap_or(5);
        let def = slime.bases.get("def").copied().unwrap_or(1);
        let (x, y) = slime
            .place
            .as_ref()
            .map(|place| (place.x, place.y))
            .unwrap_or((48, 35));
        (slime.name.clone(), x, y, hp, max_hp, atk, def)
    } else {
        ("Slime".into(), 48, 35, 12, 12, 5, 1)
    }
}

impl AuthorChoice {
    fn to_choice(&self) -> Result<crate::project::Choice, String> {
        let (_require, checks) = split_checks(&self.checks)?;
        let mut checks = checks;
        for need in _require {
            checks.push(Check::Flag {
                flag: need.flag,
                want: need.want,
            });
        }
        Ok(crate::project::Choice {
            label: self.label.clone(),
            goto: self.goto.clone(),
            sets: self.sets.clone(),
            fail: self.fail.clone(),
            checks,
            mods: mods(&self.mods),
        })
    }
}

fn split_checks(checks: &[AuthorCheck]) -> Result<(Vec<Need>, Vec<Check>), String> {
    let mut require = Vec::new();
    let mut rest = Vec::new();
    for check in checks {
        match check {
            AuthorCheck::Flag { flag, want } => require.push(Need {
                flag: flag.clone(),
                want: *want,
            }),
            AuthorCheck::Stat { stat, cmp, value } => rest.push(Check::Stat {
                stat: stat.clone(),
                cmp: Cmp::parse(cmp).ok_or_else(|| format!("bad compare {cmp}"))?,
                value: *value,
            }),
            AuthorCheck::Combat { win } => rest.push(Check::Combat { win: *win }),
        }
    }
    Ok((require, rest))
}

fn mods(list: &[AuthorMod]) -> Vec<StatMod> {
    list.iter()
        .map(|m| StatMod {
            stat: m.stat.clone(),
            delta: m.delta,
        })
        .collect()
}

fn decode_sprite(sprite: &AuthorSprite) -> Result<SpriteStrip, String> {
    let bytes = decode_data(&sprite.data)?;
    let (width, height, rgba) = decode_png_bytes(&bytes)?;
    if height != FRAME || width < FRAME || width % FRAME != 0 {
        return Err("a sprite strip is 16px tall, in 16px frames".into());
    }
    let frames = width / FRAME;
    if sprite.frames != 0 && sprite.frames != frames {
        return Err("frame count does not match the image".into());
    }
    let mut pixels = Vec::with_capacity((frames * FRAME * FRAME * 4) as usize);
    for frame in 0..frames {
        for y in 0..FRAME {
            for x in 0..FRAME {
                let sx = frame * FRAME + x;
                let i = ((y * width + sx) * 4) as usize;
                pixels.extend_from_slice(&rgba[i..i + 4]);
            }
        }
    }
    Ok(SpriteStrip {
        owner: sprite.owner.clone(),
        role: sprite.role.clone(),
        frames,
        pixels,
    })
}

fn decode_data(data: &str) -> Result<Vec<u8>, String> {
    let b64 = data.split_once("base64,").map(|(_, rest)| rest).unwrap_or(data);
    base64::engine::general_purpose::STANDARD
        .decode(b64.trim())
        .map_err(|err| err.to_string())
}

pub struct BuildOpts {
    pub game_root: PathBuf,
    pub try_cargo: bool,
}

pub fn build_portable(
    parent: &Path,
    name: &str,
    json: &str,
    opts: &BuildOpts,
) -> Result<PathBuf, String> {
    let name = name.trim();
    if name.is_empty() || name.contains('/') || name.contains('\\') || name.contains("..") {
        return Err("folder name".into());
    }
    let dest = parent.join(name);
    if dest.exists() {
        return Err("that folder already exists".into());
    }
    let project = compile_project(json)?;
    let data = dest.join("project");
    project
        .write_dir(&data)
        .map_err(|err| err.to_string())?;
    let linux = ensure_linux_binary(&opts.game_root, opts.try_cargo)?;
    let linux_name = "field-battle";
    fs::copy(&linux, dest.join(linux_name)).map_err(|err| err.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(dest.join(linux_name))
            .map_err(|err| err.to_string())?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(dest.join(linux_name), perms).map_err(|err| err.to_string())?;
    }
    let mut readme = format!(
        "\
{title}

Run ./{linux_name}

This folder is a portable game. {linux_name} is the Linux build.
The project data in ./project is what that program loads.
Nothing else needs to be installed.

See WINDOWS.txt for the Windows exe.
",
        title = project_title(json),
    );
    if let Some(exe) = windows_exe(&opts.game_root) {
        fs::copy(&exe, dest.join("field-battle.exe")).map_err(|err| err.to_string())?;
        readme.push_str("\nfield-battle.exe is the Windows build, copied beside this note.\n");
    }
    fs::write(dest.join("README.txt"), readme).map_err(|err| err.to_string())?;
    fs::write(
        dest.join("WINDOWS.txt"),
        "\
The Windows exe is produced by `cargo build --release` from the game crate with that data beside it.

From the fablespace checkout:

    cargo build --release

Copy field-battle.exe into this folder, next to the project directory.
The game loads ./project from the same folder as the exe.
",
    )
    .map_err(|err| err.to_string())?;
    Ok(dest)
}

fn project_title(json: &str) -> String {
    serde_json::from_str::<AuthorProject>(json)
        .map(|project| project.title)
        .unwrap_or_else(|_| "Fablespace".into())
}

fn windows_exe(game_root: &Path) -> Option<PathBuf> {
    let candidates = [
        game_root.join("target/x86_64-pc-windows-gnu/release/field-battle.exe"),
        game_root.join("target/release/field-battle.exe"),
    ];
    candidates.into_iter().find(|path| path.is_file())
}

fn ensure_linux_binary(game_root: &Path, try_cargo: bool) -> Result<PathBuf, String> {
    let release = game_root.join("target/release/field-battle");
    if try_cargo {
        let status = cargo_release(game_root);
        if status && release.is_file() {
            return Ok(release);
        }
    }
    if release.is_file() {
        return Ok(release);
    }
    let debug = game_root.join("target/debug/field-battle");
    if debug.is_file() {
        return Ok(debug);
    }
    Err("no Linux field-battle binary. From the game crate run: cargo build --release".into())
}

fn cargo_release(game_root: &Path) -> bool {
    let mut child = match Command::new("cargo")
        .arg("build")
        .arg("--release")
        .current_dir(game_root)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => child,
        Err(_) => return false,
    };
    let start = Instant::now();
    let limit = Duration::from_secs(180);
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return status.success(),
            Ok(None) if start.elapsed() > limit => {
                let _ = child.kill();
                let _ = child.wait();
                return false;
            }
            Ok(None) => thread::sleep(Duration::from_millis(400)),
            Err(_) => return false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::Project;

    #[test]
    fn the_sample_compiles_the_field_mira_and_the_slime_fight() {
        let json = include_str!("../author/ui/sample.json");
        let project = compile_project(json).unwrap();
        let embedded = Project::embedded();
        assert_eq!(project.tiles, embedded.tiles);
        assert_eq!(project.buildings, embedded.buildings);
        assert_eq!(project.mushrooms, embedded.mushrooms);
        assert_eq!(project.start_x, 40);
        assert_eq!(project.party[0].name, "Mira");
        assert_eq!(project.party[0].weapon_name, "Short Sword");
        assert_eq!(project.party[0].weapon_atk, 3);
        assert_eq!(project.party[0].hp, 20);
        assert!(project.party[0]
            .bases
            .iter()
            .any(|(stat, value)| stat == "strength" && *value == 4));
        assert_eq!(project.slime_name, "Slime");
        assert_eq!((project.slime_x, project.slime_y), (48, 35));
        let fight = project
            .talk
            .nodes
            .iter()
            .find(|node| node.id == "fight")
            .unwrap();
        assert!(fight
            .require
            .iter()
            .any(|need| need.flag == "slime_fight" && need.want));
        assert!(fight
            .require
            .iter()
            .any(|need| need.flag == "slime_friend" && !need.want));
        assert_eq!(fight.slot.as_deref(), Some("combat"));
        assert_eq!(fight.battle.as_ref().unwrap().entity, "slime");
        assert_eq!(fight.battle.as_ref().unwrap().map, "field");
        let greet = &project.talk.nodes[0];
        assert_eq!(greet.choices[0].sets, vec!["slime_fight".to_string()]);
        assert_eq!(greet.choices[1].sets, vec!["slime_friend".to_string()]);
        assert!(project.extras.slots.iter().any(|slot| slot.kind == "combat"));
        assert!(project.extras.tree.len() >= 2);
        assert!(project
            .extras
            .catalog
            .iter()
            .any(|item| item.name == "Short Sword"));

        let dir = std::env::temp_dir().join(format!("fablespace-bundle-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        project.write_dir(&dir).unwrap();
        let loaded = Project::read_dir(&dir).unwrap();
        assert_eq!(loaded.tiles, project.tiles);
        assert_eq!(loaded.talk.nodes[1].slot.as_deref(), Some("combat"));
        assert_eq!(loaded.extras.tree.len(), project.extras.tree.len());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_portable_build_copies_the_binary_and_names_the_windows_exe() {
        let root = std::env::temp_dir().join(format!("fablespace-root-{}", std::process::id()));
        let parent = std::env::temp_dir().join(format!("fablespace-out-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&parent);
        fs::create_dir_all(root.join("target/release")).unwrap();
        fs::write(root.join("target/release/field-battle"), b"linux-binary").unwrap();
        fs::create_dir_all(&parent).unwrap();
        let dest = build_portable(
            &parent,
            "field-game",
            include_str!("../author/ui/sample.json"),
            &BuildOpts {
                game_root: root.clone(),
                try_cargo: false,
            },
        )
        .unwrap();
        assert_eq!(fs::read(dest.join("field-battle")).unwrap(), b"linux-binary");
        assert!(dest.join("project/field.txt").is_file());
        assert!(dest.join("project/talk.txt").is_file());
        let readme = fs::read_to_string(dest.join("README.txt")).unwrap();
        assert!(readme.contains("./field-battle"));
        let windows = fs::read_to_string(dest.join("WINDOWS.txt")).unwrap();
        assert!(windows.starts_with(
            "The Windows exe is produced by `cargo build --release` from the game crate with that data beside it."
        ));
        assert!(!dest.join("field-battle.exe").exists());
        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&parent);
    }
}
