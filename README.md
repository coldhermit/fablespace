# Fablespace

A creation kit, built one tool at a time. The first tool is a pixel sprite editor.

## Sprite editor

A standalone Chromium app for drawing sprites: pens, shapes, layers, frames, onion skin, and PNG, spritesheet, and GIF export. You can also lay a semi-transparent picture over the canvas and trace it. The tracing picture is saved in the project and left out of exports.

From this folder on Windows, double-click `start.bat`. On Linux or macOS:

```sh
./start.sh
```

The first launch installs the editor's dependencies, which include Chromium. After that, the script just opens the editor.

Projects save as `.sprite.json`.
