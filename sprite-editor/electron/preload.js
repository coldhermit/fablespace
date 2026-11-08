const { contextBridge, ipcRenderer } = require("electron");

contextBridge.exposeInMainWorld("desk", {
  saveProject: (suggested, data) => ipcRenderer.invoke("save-project", { suggested, data }),
  openProject: () => ipcRenderer.invoke("open-project"),
  exportFile: (suggested, data, kind) => ipcRenderer.invoke("export-file", { suggested, data, kind }),
  pickImage: () => ipcRenderer.invoke("pick-image"),
});
