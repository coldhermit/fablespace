//! Project folders on disk.
//!
//! A project and a built game share JSON. The project keeps `game.json` and
//! `maps/` at the root. A build copies those into `data/` beside the executable.

use std::fs;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::model::{Game, Map};

#[derive(Clone, Debug)]
pub struct Paths {
    /// Folder that contains `img/` and, while playing, `save/`.
    pub root: PathBuf,
    /// Folder that contains `game.json` and `maps/`.
    pub data: PathBuf,
}

impl Paths {
    pub fn game_file(&self) -> PathBuf {
        self.data.join("game.json")
    }

    pub fn maps_dir(&self) -> PathBuf {
        self.data.join("maps")
    }

    pub fn map_file(&self, id: u32) -> PathBuf {
        self.maps_dir().join(format!("{id}.json"))
    }

    pub fn img_dir(&self) -> PathBuf {
        self.root.join("img")
    }

    pub fn image(&self, folder: &str, name: &str) -> PathBuf {
        self.img_dir().join(folder).join(format!("{name}.png"))
    }

    pub fn save_dir(&self) -> PathBuf {
        self.root.join("save")
    }

    pub fn save_file(&self, slot: usize) -> PathBuf {
        self.save_dir().join(format!("slot{:02}.json", slot + 1))
    }
}

/// `--project` points at a project root. No flag means the folder that holds
/// the executable, which is a build when `data/game.json` is there.
pub fn resolve(project: Option<&Path>, exe_dir: &Path) -> Result<Paths, String> {
    if let Some(root) = project {
        let root = root.to_path_buf();
        if root.join("game.json").is_file() {
            return Ok(Paths {
                data: root.clone(),
                root,
            });
        }
        if root.join("data/game.json").is_file() {
            return Ok(Paths {
                data: root.join("data"),
                root,
            });
        }
        return Err(format!(
            "no game.json in {} or {}/data",
            root.display(),
            root.display()
        ));
    }
    if exe_dir.join("data/game.json").is_file() {
        return Ok(Paths {
            root: exe_dir.to_path_buf(),
            data: exe_dir.join("data"),
        });
    }
    Err("no project was given and no data/game.json sits beside the executable".into())
}

pub fn load_game(paths: &Paths) -> Result<Game, String> {
    let mut game: Game = read_json(&paths.game_file())?;
    game.normalize();
    Ok(game)
}

pub fn load_maps(paths: &Paths) -> Result<Vec<Map>, String> {
    let dir = paths.maps_dir();
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut files: Vec<PathBuf> = fs::read_dir(&dir)
        .map_err(|e| format!("read {}: {e}", dir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("json"))
        .collect();
    files.sort();
    files.into_iter().map(|p| read_json(&p)).collect()
}

pub fn save_project(root: &Path, game: &Game, maps: &[Map]) -> Result<(), String> {
    let paths = Paths {
        root: root.to_path_buf(),
        data: root.to_path_buf(),
    };
    write_json(&paths.game_file(), game)?;
    fs::create_dir_all(paths.maps_dir()).map_err(|e| e.to_string())?;
    let mut keep = Vec::new();
    for map in maps {
        let path = paths.map_file(map.id);
        write_json(&path, map)?;
        keep.push(path);
    }
    if let Ok(entries) = fs::read_dir(paths.maps_dir()) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") && !keep.contains(&path) {
                let _ = fs::remove_file(path);
            }
        }
    }
    Ok(())
}

pub fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("parse {}: {e}", path.display()))
}

pub fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("create {}: {e}", parent.display()))?;
    }
    let text = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, text).map_err(|e| format!("write {}: {e}", tmp.display()))?;
    fs::rename(&tmp, path).map_err(|e| format!("rename {}: {e}", path.display()))?;
    Ok(())
}

pub fn sanitize_title(title: &str) -> String {
    let cleaned: String = title
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => ' ',
            c if c.is_control() => ' ',
            c => c,
        })
        .collect();
    let trimmed = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
    if trimmed.is_empty() {
        "Game".into()
    } else {
        trimmed
    }
}
