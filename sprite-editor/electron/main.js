const { app, BrowserWindow, dialog, ipcMain } = require("electron");
const fs = require("fs");
const path = require("path");

const filters = {
  project: [{ name: "Sprite", extensions: ["sprite.json", "json"] }],
  png: [{ name: "PNG", extensions: ["png"] }],
  gif: [{ name: "GIF", extensions: ["gif"] }],
  image: [{ name: "Image", extensions: ["png", "gif", "jpg", "jpeg", "webp", "bmp"] }],
};

function createWindow() {
  const win = new BrowserWindow({
    width: 1400,
    height: 900,
    minWidth: 960,
    minHeight: 640,
    backgroundColor: "#161616",
    title: "Sprite Editor",
    webPreferences: {
      preload: path.join(__dirname, "preload.js"),
      contextIsolation: true,
      nodeIntegration: false,
      sandbox: true,
    },
  });
  win.setMenuBarVisibility(false);
  win.loadFile(path.join(__dirname, "..", "editor", "index.html"));
  return win;
}

function writeBase64(filePath, dataUrlOrB64) {
  const b64 = String(dataUrlOrB64).replace(/^data:[^,]*,/, "");
  fs.writeFileSync(filePath, Buffer.from(b64, "base64"));
}

ipcMain.handle("save-project", async (_event, { suggested, data }) => {
  const { filePath, canceled } = await dialog.showSaveDialog({
    defaultPath: suggested || "sprite.sprite.json",
    filters: filters.project,
  });
  if (canceled || !filePath) return null;
  fs.writeFileSync(filePath, data, "utf8");
  return filePath;
});

ipcMain.handle("open-project", async () => {
  const { filePaths, canceled } = await dialog.showOpenDialog({
    filters: filters.project,
    properties: ["openFile"],
  });
  if (canceled || !filePaths[0]) return null;
  return { path: filePaths[0], text: fs.readFileSync(filePaths[0], "utf8") };
});

ipcMain.handle("export-file", async (_event, { suggested, data, kind }) => {
  const { filePath, canceled } = await dialog.showSaveDialog({
    defaultPath: suggested,
    filters: kind === "gif" ? filters.gif : filters.png,
  });
  if (canceled || !filePath) return null;
  writeBase64(filePath, data);
  return filePath;
});

ipcMain.handle("pick-image", async () => {
  const { filePaths, canceled } = await dialog.showOpenDialog({
    filters: filters.image,
    properties: ["openFile"],
  });
  if (canceled || !filePaths[0]) return null;
  const filePath = filePaths[0];
  const buf = fs.readFileSync(filePath);
  const ext = path.extname(filePath).slice(1).toLowerCase() || "png";
  const mime = ext === "jpg" || ext === "jpeg" ? "image/jpeg" : ext === "webp" ? "image/webp" : ext === "gif" ? "image/gif" : ext === "bmp" ? "image/bmp" : "image/png";
  return { name: path.basename(filePath), dataUrl: `data:${mime};base64,${buf.toString("base64")}` };
});

app.whenReady().then(() => {
  const win = createWindow();
  if (process.env.SPRITE_SMOKE) {
    win.webContents.on("did-finish-load", async () => {
      const out = process.env.SPRITE_SMOKE;
      try {
        await win.webContents.executeJavaScript("window.__smoke && window.__smoke()");
        const image = await win.webContents.capturePage();
        fs.mkdirSync(path.dirname(out), { recursive: true });
        fs.writeFileSync(out, image.toPNG());
        app.exit(0);
      } catch (err) {
        fs.mkdirSync(path.dirname(out), { recursive: true });
        fs.writeFileSync(out + ".error.txt", String(err && err.stack || err));
        app.exit(1);
      }
    });
  }
  app.on("activate", () => {
    if (BrowserWindow.getAllWindows().length === 0) createWindow();
  });
});

app.on("window-all-closed", () => {
  if (process.platform !== "darwin") app.quit();
});
