# Build the game and the author. The game exe is copied next to project\.
# Double-click build.bat, or run this file from PowerShell.
$ErrorActionPreference = "Stop"
$Root = $PSScriptRoot
Set-Location $Root

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Error "Rust is not on PATH. Install it from https://rustup.rs, open a new terminal, and run this again."
}
if (-not (Get-Command rustup -ErrorAction SilentlyContinue)) {
    Write-Error "rustup is not on PATH. Install Rust from https://rustup.rs and run this again."
}

rustup show | Out-Host
$env:CARGO_TARGET_DIR = Join-Path $Root "target"

# A Linux cross-compile exports this linker. It is the wrong name for the
# gcc that rustup's Windows GNU toolchain ships.
if ($env:CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER -eq "x86_64-w64-mingw32-gcc") {
    if (-not (Get-Command x86_64-w64-mingw32-gcc -ErrorAction SilentlyContinue)) {
        Remove-Item Env:CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER
    }
}

function Get-VcVars {
    if (Get-Command link.exe -ErrorAction SilentlyContinue) {
        return "path"
    }
    $vswhere = Join-Path ${env:ProgramFiles(x86)} "Microsoft Visual Studio\Installer\vswhere.exe"
    if (-not (Test-Path $vswhere)) {
        return $null
    }
    $install = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
    if (-not $install) {
        return $null
    }
    $vcvars = Join-Path $install "VC\Auxiliary\Build\vcvars64.bat"
    if (Test-Path $vcvars) {
        return $vcvars
    }
    return $null
}

$prefix = @()
$vcvars = $null
$hostLine = (& rustc -vV | Select-String "^host:").Line
$hostTriple = $hostLine.Split(" ", [System.StringSplitOptions]::RemoveEmptyEntries)[-1]

if ($hostTriple -like "*windows-msvc*") {
    $vcvars = Get-VcVars
    if (-not $vcvars) {
        $channel = "1.99.1"
        $toolchainFile = Join-Path $Root "rust-toolchain.toml"
        if (Test-Path $toolchainFile) {
            $match = Select-String -Path $toolchainFile -Pattern 'channel\s*=\s*"([^"]+)"' | Select-Object -First 1
            if ($match) {
                $channel = $match.Matches[0].Groups[1].Value
            }
        }
        $gnu = "$channel-x86_64-pc-windows-gnu"
        Write-Host "No MSVC linker on PATH. Installing $gnu (it includes gcc)..."
        & rustup toolchain install $gnu --profile minimal
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
        $prefix = @("+$gnu")
    }
}

function Invoke-Cargo([string[]]$cargoArgs) {
    if ($vcvars -and $vcvars -ne "path") {
        $joined = ($cargoArgs -join " ")
        cmd /c "call `"$vcvars`" >nul && cargo $joined"
    } else {
        & cargo @cargoArgs
    }
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}

Write-Host "Building the game..."
Invoke-Cargo ($prefix + @("build", "--release"))

$game = Join-Path $env:CARGO_TARGET_DIR "release\field-battle.exe"
if (-not (Test-Path $game)) {
    Write-Error "cargo finished but $game is missing."
}
Copy-Item -Force $game (Join-Path $Root "field-battle.exe")
Write-Host "Ready: $(Join-Path $Root 'field-battle.exe')"

Write-Host "Building the author..."
Invoke-Cargo ($prefix + @("build", "--release", "--manifest-path", "author/src-tauri/Cargo.toml"))
$author = Join-Path $env:CARGO_TARGET_DIR "release\fablespace-author.exe"
if (-not (Test-Path $author)) {
    Write-Error "cargo finished but $author is missing."
}
Copy-Item -Force $author (Join-Path $Root "fablespace-author.exe")
Write-Host "Ready: $(Join-Path $Root 'fablespace-author.exe')"
Write-Host "The game loads .\project from the same folder as field-battle.exe."
