# fablespace

A grass field, a town, and one conversation. The author in `author/` is a desktop window that writes this game.

## Play

On Windows, double-click `build.bat`. On Linux or macOS, run `./build.sh`.
Either one builds the release game and the author, and copies the game next to `project/`.

```
cargo run --release
```

The game loads `project/` beside the executable, then `./project`.

## Author

The author uses Tauri: Rust, plus the system webview (WebView2 on Windows). `rust-toolchain.toml` selects Rust 1.99.1, which the current Tauri crates need.

```
cargo run --manifest-path author/src-tauri/Cargo.toml
```

Save stores the author project only. It does not build the game.

Build asks for a parent folder and a new folder name, then writes a portable game there: the pixel executable and the project data it loads. Nothing else is installed.
