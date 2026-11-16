# Fablespace

Fablespace is a small tile RPG maker. Paint a map, write events, fill in the database, then play. A build is one folder you can copy onto another machine of the same operating system.

The editor is an iced window. The game draws with wgpu.

## Maker

```
cargo run -p fablespace-maker
```

Rust 1.85 or newer. **New Project** writes a meadow you can edit. **Open Project** opens the folder that contains `game.json`. **Open Fable Hollow** opens the sample in `sample/`.

**Save** writes the project. **Playtest** saves, builds the game, and launches it. **Build** saves, builds the release game, and copies it into `dist/` inside the project.

## Play

```
cargo run -p fablespace-runtime -- --project sample
```

- Arrows or WASD move.
- Z, Enter, or Space confirms.
- X or Backspace cancels.
- Esc opens the menu, and closes a window when one is open.
- Hold Shift to dash.

The menu is items, skills, equipment, status, save, and end game. Battles are turn based: attack, guard, skills, and items. Save files are `save/slot01.json` through `save/slot04.json` next to the project, or next to the built game.

## Build folder

```
dist/Fable Hollow/
  Fable Hollow
  data/game.json
  data/maps/
  img/characters/
  img/faces/
  img/tilesets/
  img/enemies/
  img/pictures/
  img/titles/
```

On Windows the game file ends in `.exe`. Copy the whole folder. It does not need the maker, Cargo, or the original project path. It does use the graphics driver already on the machine: Vulkan on Linux, Direct3D 12 on Windows, and Metal on macOS.

## Project files

`game.json` is the database and the system settings. Each map is `maps/<id>.json`. Pictures are PNGs under `img/`. A build copies that JSON into `data/` unchanged.

Tiles are 32 pixels. The view is 17 by 13 tiles. The lower layer is the ground. The upper layer is decoration such as flowers and trees. An autotile joins the same tile on the four neighboring sides. Its sixteen shapes occupy a 4 by 4 block, and the id you paint is the top-left of that block.

## Layout

- `fablespace-data` holds the project types the editor and the game share.
- `fablespace-maker` is the editor.
- `fablespace-runtime` is the game.

The extra crates are serde, an image reader, iced, and wgpu. There is no plugin loader and no scripting language. A physical hit is `power + attack × 4 − defense × 2`. A magical hit uses magic attack and magic defense the same way.

## Fable Hollow

Lark starts on the south path of the meadow. Talk to Mara. Read the notice board. Open the chest for a berry. Walk into the slime at the east end of the path.
