//! Project files on disk, and the cargo commands behind Playtest and Build.

use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use fablespace_data::sample;
use fablespace_data::{install, project, Game, Map};

use crate::art;

pub fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

pub fn write_project(root: &Path, title: Option<&str>) -> Result<(), String> {
    let (mut game, maps) = sample::fable_hollow();
    if let Some(title) = title.map(str::trim).filter(|t| !t.is_empty()) {
        game.system.game_title = title.to_string();
    }
    project::save_project(root, &game, &maps)?;
    art::write_all(root)
}

pub fn load_project(root: &Path) -> Result<(Game, Vec<Map>), String> {
    let paths = fablespace_data::Paths {
        root: root.to_path_buf(),
        data: root.to_path_buf(),
    };
    if !paths.game_file().is_file() {
        return Err(format!("no game.json in {}", root.display()));
    }
    let game = project::load_game(&paths)?;
    let maps = project::load_maps(&paths)?;
    Ok((game, maps))
}

pub fn save_project(root: &Path, game: &Game, maps: &[Map]) -> Result<(), String> {
    project::save_project(root, game, maps)
}

pub enum JobKind {
    Playtest { project: PathBuf },
    Build { project: PathBuf },
}

pub struct Job {
    pub child: Child,
    pub log_path: PathBuf,
    pub kind: JobKind,
}

pub fn start_cargo(project: &Path, release: bool) -> Result<Job, String> {
    let log_path = project.join(".fablespace-build.log");
    let log = File::create(&log_path).map_err(|e| format!("log: {e}"))?;
    let mut cmd = Command::new("cargo");
    cmd.current_dir(workspace_root())
        .arg("build")
        .arg("-p")
        .arg("fablespace-runtime")
        .stdout(Stdio::from(log.try_clone().map_err(|e| e.to_string())?))
        .stderr(Stdio::from(log));
    if release {
        cmd.arg("--release");
    }
    let child = cmd.spawn().map_err(|e| format!("cargo: {e}"))?;
    Ok(Job {
        child,
        log_path,
        kind: if release {
            JobKind::Build {
                project: project.to_path_buf(),
            }
        } else {
            JobKind::Playtest {
                project: project.to_path_buf(),
            }
        },
    })
}

pub fn runtime_bin(release: bool) -> PathBuf {
    let folder = if release { "release" } else { "debug" };
    workspace_root().join("target").join(folder).join(format!(
        "fablespace-runtime{}",
        std::env::consts::EXE_SUFFIX
    ))
}

pub fn spawn_playtest(project: &Path) -> Result<(), String> {
    let mut child = Command::new(runtime_bin(false))
        .arg("--project")
        .arg(project)
        .spawn()
        .map_err(|e| format!("playtest: {e}"))?;
    std::thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(())
}

pub fn import_png(project: &Path, category: &str, source: &Path) -> Result<String, String> {
    if !category.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err("invalid image folder".into());
    }
    let stem = source
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("image");
    let mut name: String = stem
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
        .collect();
    if name.is_empty() {
        name = "image".into();
    }
    let dest_dir = project.join("img").join(category);
    std::fs::create_dir_all(&dest_dir).map_err(|e| e.to_string())?;
    std::fs::copy(source, dest_dir.join(format!("{name}.png"))).map_err(|e| e.to_string())?;
    Ok(name)
}

pub fn assemble(project: &Path) -> Result<PathBuf, String> {
    let dist = project.join("dist");
    install::assemble(&runtime_bin(true), project, &dist)
}

pub fn log_tail(path: &Path) -> String {
    let Ok(text) = std::fs::read_to_string(path) else {
        return String::new();
    };
    let mut lines: Vec<&str> = text.lines().rev().take(12).collect();
    lines.reverse();
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_project_roundtrips() {
        let dir = std::env::temp_dir().join(format!("fablespace-maker-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        write_project(&dir, Some("Lantern Path")).unwrap();
        let (game, maps) = load_project(&dir).unwrap();
        assert_eq!(game.system.game_title, "Lantern Path");
        assert_eq!(maps.len(), 1);
        assert!(dir.join("img/tilesets/meadow.png").is_file());
        assert!(dir.join("img/characters/hero.png").is_file());
        assert!(dir.join("img/enemies/slime.png").is_file());
        assert!(dir.join("img/titles/title.png").is_file());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
