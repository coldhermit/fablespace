#!/bin/sh
# Build the pixel game from this folder.
# On Linux, also cross-build field-battle.exe when the mingw gcc is installed.
set -eu
cd "$(dirname "$0")"
root=$(pwd)

if ! command -v cargo >/dev/null 2>&1; then
  echo "Rust is not on PATH. Install it from https://rustup.rs and run this again." >&2
  exit 1
fi

rustup show
export CARGO_TARGET_DIR="$root/target"

echo "Building the game..."
cargo build --release

host=$(rustc -vV | sed -n 's/^host: //p')
case "$host" in
  *windows*) game="$CARGO_TARGET_DIR/release/field-battle.exe" ;;
  *) game="$CARGO_TARGET_DIR/release/field-battle" ;;
esac
cp -f "$game" "$root/$(basename "$game")"
echo "Ready: $root/$(basename "$game")"

if command -v x86_64-w64-mingw32-gcc >/dev/null 2>&1; then
  echo "Building field-battle.exe..."
  rustup target add x86_64-pc-windows-gnu
  CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc \
    cargo build --release --target x86_64-pc-windows-gnu
  cp -f "$CARGO_TARGET_DIR/x86_64-pc-windows-gnu/release/field-battle.exe" "$root/field-battle.exe"
  echo "Ready: $root/field-battle.exe"
fi

echo "The game loads ./project from the same folder as the executable."
