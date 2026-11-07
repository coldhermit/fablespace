#!/bin/sh
# Build the Twine-style author. The game is what the author's Build button writes.
set -eu
cd "$(dirname "$0")"
root=$(pwd)

if ! command -v cargo >/dev/null 2>&1; then
  echo "Rust is not on PATH. Install it from https://rustup.rs and run this again." >&2
  exit 1
fi

rustup show
export CARGO_TARGET_DIR="$root/target"

echo "Building the author..."
cargo build --release --manifest-path author/src-tauri/Cargo.toml

host=$(rustc -vV | sed -n 's/^host: //p')
case "$host" in
  *windows*) author="$CARGO_TARGET_DIR/release/fablespace-author.exe" ;;
  *) author="$CARGO_TARGET_DIR/release/fablespace-author" ;;
esac
cp -f "$author" "$root/$(basename "$author")"
echo "Ready: $root/$(basename "$author")"
echo "Save stores the project. Build, inside the author, writes the game."
