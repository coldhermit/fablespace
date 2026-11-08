# Fablespace

A creation kit, built one tool at a time. The first tool is a pixel sprite editor.

## Sprite editor

A Chromium desktop app for drawing sprites: pens, shapes, layers, frames, onion skin, and PNG, spritesheet, and GIF export. Each frame keeps its own tracing picture. The eyedropper reads that picture's own color. 100% scale fits it inside the sprite, and Move trace unlocks both dragging and scaling. Hold Space and drag to move the canvas. Tracing pictures are saved with the project and left out of exports.

On Windows, download the zip from the [latest release](https://github.com/coldhermit/fablespace/releases), unzip it, and run `SpriteEditor.exe`. That is the app itself. Closing the window quits it. If the drawing has unsaved changes, it asks first.

From a checkout, `start.bat` opens that exe. The first time, if the exe is not there yet, the script builds it and then opens it.

Projects save as `.sprite.json`.
