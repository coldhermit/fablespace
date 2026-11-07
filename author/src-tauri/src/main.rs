//! Desktop author. The window is the system webview: WebView2 on Windows,
//! WebKitGTK on Linux. Save writes the author project only. Build writes a
//! portable copy of the pixel game.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use field_battle::bundle::{build_portable, BuildOpts};
use std::fs;
use std::path::PathBuf;

fn game_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."))
}

#[tauri::command]
fn save_project(path: String, json: String) -> Result<(), String> {
    serde_json::from_str::<serde_json::Value>(&json).map_err(|err| err.to_string())?;
    let path = PathBuf::from(path.trim());
    if path.as_os_str().is_empty() {
        return Err("need a file".into());
    }
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|err| err.to_string())?;
        }
    }
    fs::write(path, json).map_err(|err| err.to_string())
}

#[tauri::command]
fn build_game(parent: String, name: String, json: String) -> Result<String, String> {
    let dest = build_portable(
        PathBuf::from(parent.trim()).as_path(),
        name.trim(),
        &json,
        &BuildOpts {
            game_root: game_root(),
            try_cargo: true,
        },
    )?;
    Ok(dest.display().to_string())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![save_project, build_game])
        .run(tauri::generate_context!())
        .expect("failed to open the author");
}
