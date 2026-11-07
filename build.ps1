# Build field-battle.exe in this folder.
# Requires Rust: https://rustup.rs  (the default Windows toolchain is fine)
$ErrorActionPreference = "Stop"
$root = $PSScriptRoot
Set-Location $root

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Error "Rust is not on PATH. Install it from https://rustup.rs, open a new terminal, and run this script again."
}

if (-not (Test-Path (Join-Path $root "Cargo.toml"))) {
    if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
        Write-Error "This folder has no Cargo.toml, and git is not on PATH."
    }
    $tmp = Join-Path $env:TEMP "fablespace-clone"
    if (Test-Path $tmp) { Remove-Item -Recurse -Force $tmp }
    git clone https://github.com/coldhermit/fablespace.git $tmp
    Get-ChildItem -Force $tmp | ForEach-Object {
        $dest = Join-Path $root $_.Name
        if (Test-Path $dest) { Remove-Item -Recurse -Force $dest }
        Move-Item $_.FullName $dest
    }
    Remove-Item -Force $tmp
}

# The repo config points target-dir at a Linux path. Override it here.
$env:CARGO_TARGET_DIR = Join-Path $root "target"
cargo build --release
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$built = Join-Path $env:CARGO_TARGET_DIR "release\field-battle.exe"
$out = Join-Path $root "field-battle.exe"
Copy-Item -Force $built $out
Write-Host "Ready: $out"
