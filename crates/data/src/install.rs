//! Copy a compiled runtime and a project's files into one folder.

use std::fs;
use std::path::{Path, PathBuf};

use crate::model::Game;
use crate::project::{sanitize_title, Paths};

pub fn assemble(
    runtime_bin: &Path,
    project_root: &Path,
    dist_parent: &Path,
) -> Result<PathBuf, String> {
    if !runtime_bin.is_file() {
        return Err(format!(
            "runtime binary not found at {}",
            runtime_bin.display()
        ));
    }
    let paths = Paths {
        root: project_root.to_path_buf(),
        data: project_root.to_path_buf(),
    };
    let game = crate::project::load_game(&paths)?;
    let title = sanitize_title(&game.system.game_title);
    let dest = dist_parent.join(&title);
    if dest.exists() {
        fs::remove_dir_all(&dest).map_err(|e| format!("clear {}: {e}", dest.display()))?;
    }
    fs::create_dir_all(dest.join("data").join("maps")).map_err(|e| e.to_string())?;
    fs::create_dir_all(dest.join("img")).map_err(|e| e.to_string())?;
    let exe_name = format!("{title}{}", std::env::consts::EXE_SUFFIX);
    let exe_path = dest.join(&exe_name);
    fs::copy(runtime_bin, &exe_path).map_err(|e| format!("copy runtime: {e}"))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&exe_path)
            .map_err(|e| e.to_string())?
            .permissions();
        perms.set_mode(perms.mode() | 0o755);
        fs::set_permissions(&exe_path, perms).map_err(|e| e.to_string())?;
    }
    fs::copy(paths.game_file(), dest.join("data").join("game.json")).map_err(|e| e.to_string())?;
    copy_tree(&paths.maps_dir(), &dest.join("data").join("maps"))?;
    copy_tree(&paths.img_dir(), &dest.join("img"))?;
    let _ = game;
    Ok(dest)
}

fn copy_tree(from: &Path, to: &Path) -> Result<(), String> {
    if !from.exists() {
        return Ok(());
    }
    fs::create_dir_all(to).map_err(|e| e.to_string())?;
    for entry in fs::read_dir(from).map_err(|e| format!("read {}: {e}", from.display()))? {
        let entry = entry.map_err(|e| e.to_string())?;
        let src = entry.path();
        let dst = to.join(entry.file_name());
        if src.is_dir() {
            copy_tree(&src, &dst)?;
        } else {
            fs::copy(&src, &dst).map_err(|e| format!("copy {}: {e}", src.display()))?;
        }
    }
    Ok(())
}

/// The title is part of the install folder name. Load it without the rest of the copy.
pub fn title_of(project_root: &Path) -> Result<String, String> {
    let game: Game = crate::project::read_json(&project_root.join("game.json"))?;
    Ok(sanitize_title(&game.system.game_title))
}
