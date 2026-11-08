(() => {
  const P = window.Pixels;
  const $ = (id) => document.getElementById(id);
  const TRANSPARENT = [0, 0, 0, 0];
  const PALETTE = [
    [0, 0, 0, 0],
    [20, 16, 16, 255],
    [245, 240, 230, 255],
    [140, 128, 116, 255],
    [88, 78, 70, 255],
    [196, 58, 42, 255],
    [227, 123, 45, 255],
    [240, 208, 96, 255],
    [62, 148, 78, 255],
    [48, 126, 176, 255],
    [46, 64, 150, 255],
    [126, 62, 168, 255],
    [214, 92, 140, 255],
    [120, 74, 42, 255],
    [236, 196, 156, 255],
  ];

  const state = {
    name: "sprite",
    width: 32,
    height: 32,
    fps: 8,
    layers: [],
    frame: 0,
    layer: 0,
    tool: "pen",
    size: 1,
    primary: [20, 16, 16, 255],
    secondary: [245, 240, 230, 255],
    palette: PALETTE.map((c) => c.slice()),
    zoom: 16,
    panX: 0,
    panY: 0,
    grid: true,
    tile: false,
    onion: false,
    dim: true,
    playing: true,
    traces: [],
    trace: { dataUrl: null, image: null, opacity: 0.45, visible: true, scale: 1, x: 0, y: 0 },
    traceMove: false,
    selection: null,
    clipboard: null,
    undo: [],
    redo: [],
    dirty: false,
    path: null,
  };

  let gesture = null;
  let colorTarget = "primary";
  let pan = null;
  let hand = false;
  let hover = null;
  let previewTick = 0;
  let lastPreview = 0;
  const view = $("view");
  const canvas = $("canvas");
  const ctx = canvas.getContext("2d");
  const preview = $("preview");
  const pctx = preview.getContext("2d");

  function blank() {
    return new Uint8ClampedArray(state.width * state.height * 4);
  }

  function makeLayer(name, frames) {
    const count = Math.max(1, state.layers[0] ? state.layers[0].frames.length : 1);
    return {
      name,
      opacity: 1,
      visible: true,
      frames: frames || Array.from({ length: count }, blank),
    };
  }

  function frameCount() {
    return state.layers[0].frames.length;
  }

  function blankTrace() {
    return { dataUrl: null, image: null, opacity: 0.45, visible: true, scale: 1, x: 0, y: 0 };
  }

  function copyTrace(trace) {
    return {
      dataUrl: trace.dataUrl,
      image: trace.image,
      opacity: trace.opacity,
      visible: trace.visible,
      scale: trace.scale,
      x: trace.x,
      y: trace.y,
    };
  }

  function syncTraceCount() {
    if (!state.layers[0]) return;
    const count = state.layers[0].frames.length;
    while (state.traces.length < count) state.traces.push(blankTrace());
    if (state.traces.length > count) state.traces.length = count;
    if (state.frame >= state.traces.length) state.frame = Math.max(0, state.traces.length - 1);
    state.trace = state.traces[state.frame];
  }

  function currentBuffer() {
    return state.layers[state.layer].frames[state.frame];
  }

  function markDirty() {
    state.dirty = true;
    $("doc-name").classList.add("dirty");
  }

  function snapshot() {
    return {
      name: state.name,
      width: state.width,
      height: state.height,
      fps: state.fps,
      frame: state.frame,
      layer: state.layer,
      layers: state.layers.map((layer) => ({
        name: layer.name,
        opacity: layer.opacity,
        visible: layer.visible,
        frames: layer.frames.map((frame) => P.clone(frame)),
      })),
      traces: state.traces.map((trace) => copyTrace(trace)),
      selection: state.selection
        ? {
            x: state.selection.x,
            y: state.selection.y,
            w: state.selection.w,
            h: state.selection.h,
            floating: state.selection.floating,
            pixels: P.clone(state.selection.pixels),
          }
        : null,
    };
  }

  function restore(snap) {
    state.name = snap.name;
    state.width = snap.width;
    state.height = snap.height;
    state.fps = snap.fps;
    state.frame = snap.frame;
    state.layer = snap.layer;
    state.layers = snap.layers.map((layer) => ({
      name: layer.name,
      opacity: layer.opacity,
      visible: layer.visible,
      frames: layer.frames.map((frame) => P.clone(frame)),
    }));
    const previous = state.traces;
    state.traces = (snap.traces || []).map((trace) => {
      const same = previous.find((item) => item.dataUrl && item.dataUrl === trace.dataUrl && item.image);
      return {
        dataUrl: trace.dataUrl || null,
        image: same ? same.image : null,
        opacity: trace.opacity == null ? 0.45 : trace.opacity,
        visible: trace.visible !== false,
        scale: trace.scale == null ? 1 : trace.scale,
        x: trace.x || 0,
        y: trace.y || 0,
      };
    });
    syncTraceCount();
    state.traces.forEach((trace) => {
      if (trace.dataUrl && !trace.image) loadTraceInto(trace, trace.dataUrl, false);
    });
    state.selection = snap.selection
      ? {
          x: snap.selection.x,
          y: snap.selection.y,
          w: snap.selection.w,
          h: snap.selection.h,
          floating: snap.selection.floating,
          pixels: P.clone(snap.selection.pixels),
        }
      : null;
    syncTraceControls();
    $("fps").value = state.fps;
    buildLayers();
    buildFrames();
    paintChrome();
    render();
  }

  function pushUndo() {
    state.undo.push(snapshot());
    if (state.undo.length > 50) state.undo.shift();
    state.redo.length = 0;
    markDirty();
  }

  function undo() {
    if (!state.undo.length) return;
    state.redo.push(snapshot());
    restore(state.undo.pop());
  }

  function redo() {
    if (!state.redo.length) return;
    state.undo.push(snapshot());
    restore(state.redo.pop());
  }

  function resetDocument(w, h, name) {
    state.width = w;
    state.height = h;
    state.name = name || "sprite";
    state.frame = 0;
    state.layer = 0;
    state.layers = [makeLayer("Layer 1", [blank()])];
    state.traces = [blankTrace()];
    state.trace = state.traces[0];
    traceSample = null;
    state.selection = null;
    state.undo = [];
    state.redo = [];
    state.dirty = false;
    state.path = null;
    $("doc-name").textContent = state.name;
    $("doc-name").classList.remove("dirty");
    buildLayers();
    buildFrames();
    fit();
    render();
  }

  function colorFor(button, tool) {
    if (tool === "eraser") return TRANSPARENT;
    if (tool === "dither" && button === 2) return [state.secondary, state.primary];
    return button === 2 ? state.secondary : state.primary;
  }

  function targets(evt) {
    const allLayers = evt.ctrlKey || evt.metaKey;
    const allFrames = evt.shiftKey;
    const layers = allLayers ? state.layers.map((_, i) => i) : [state.layer];
    const frames = allFrames ? state.layers[0].frames.map((_, i) => i) : [state.frame];
    const list = [];
    for (const li of layers) for (const fi of frames) list.push(state.layers[li].frames[fi]);
    return list;
  }

  function mirrorPoints(points, evt) {
    const both = evt.shiftKey;
    const horizontal = evt.ctrlKey || evt.metaKey;
    const out = points.slice();
    const add = (x, y) => out.push([x, y]);
    if (!horizontal || both) {
      for (const [x, y] of points) add(state.width - 1 - x, y);
    }
    if (horizontal || both) {
      for (const [x, y] of points) add(x, state.height - 1 - y);
    }
    if (both) {
      for (const [x, y] of points) add(state.width - 1 - x, state.height - 1 - y);
    }
    return out;
  }

  function paintPoints(buf, points, size, color) {
    if (Array.isArray(color[0])) {
      const o = P.stampOrigin(size);
      for (const [x, y] of points) {
        for (let dy = 0; dy < size; dy++) {
          for (let dx = 0; dx < size; dx++) {
            const px = x + o + dx;
            const py = y + o + dy;
            if (px < 0 || py < 0 || px >= state.width || py >= state.height) continue;
            const ink = ((px + py) & 1) === 0 ? color[0] : color[1];
            P.set(buf, px, py, state.width, ink);
          }
        }
      }
      return;
    }
    for (const [x, y] of points) P.stamp(buf, state.width, state.height, x, y, size, color);
  }

  function lightenPoints(buf, points, size, amount, once) {
    const o = P.stampOrigin(size);
    for (const [x, y] of points) {
      for (let dy = 0; dy < size; dy++) {
        for (let dx = 0; dx < size; dx++) {
          const px = x + o + dx;
          const py = y + o + dy;
          const key = px + "," + py;
          if (once && gesture.once.has(key)) continue;
          if (once) gesture.once.add(key);
          P.lightenAt(buf, state.width, state.height, px, py, amount);
        }
      }
    }
  }

function applyTool(evt, x, y, first) {
  const tool = state.tool;
  const bufs = gesture.bufs;
  if ((tool === "bucket" || tool === "magic") && !first) return;
  if (tool === "pen" || tool === "eraser" || tool === "dither" || tool === "lighten" || tool === "mirror") {
      let pts = P.line(gesture.last[0], gesture.last[1], x, y);
      if (first) pts = [[x, y]];
      if (tool === "mirror") pts = mirrorPoints(pts, evt);
      const color = colorFor(gesture.button, tool);
      const amount = (evt.shiftKey ? 0.22 : 0.08) * (evt.ctrlKey || evt.metaKey ? -1 : 1);
      for (const buf of bufs) {
        if (tool === "lighten") lightenPoints(buf, pts, state.size, amount, evt.shiftKey);
        else paintPoints(buf, pts, state.size, color);
      }
      gesture.last = [x, y];
      return;
    }
    for (const saved of gesture.saved) saved.buf.set(saved.data);
    if (tool === "line") {
      let x1 = x;
      let y1 = y;
      if (evt.shiftKey) [x1, y1] = P.snapLine(gesture.x0, gesture.y0, x, y);
      gesture.x1 = x1;
      gesture.y1 = y1;
      const pts = P.line(gesture.x0, gesture.y0, x1, y1);
      const color = colorFor(gesture.button, tool);
      for (const buf of bufs) paintPoints(buf, pts, state.size, color);
    } else if (tool === "rect" || tool === "ellipse") {
      const boxed = P.box(gesture.x0, gesture.y0, x, y, evt.shiftKey);
      gesture.box = boxed;
      const pts = tool === "rect" ? P.rectPoints(boxed[0], boxed[1], boxed[2], boxed[3], state.size) : P.ellipsePoints(boxed[0], boxed[1], boxed[2], boxed[3], state.size);
      const color = colorFor(gesture.button, tool);
      for (const buf of bufs) P.plot(buf, state.width, state.height, pts, color);
    } else if (tool === "move") {
      const dx = x - gesture.x0;
      const dy = y - gesture.y0;
      gesture.shift = [dx, dy];
      gesture.saved.forEach((saved) => {
        saved.buf.set(P.shift(saved.data, state.width, state.height, dx, dy, evt.altKey));
      });
    } else if (tool === "bucket") {
      const color = colorFor(gesture.button, tool);
      for (const buf of bufs) P.flood(buf, state.width, state.height, x, y, color);
    } else if (tool === "magic") {
      const color = colorFor(gesture.button, tool);
      const from = P.get(bufs[0], x, y, state.width);
      for (const buf of bufs) P.replaceColor(buf, from, color);
    }
  }

  function commitSelection() {
    if (!state.selection || !state.selection.floating) {
      state.selection = null;
      return;
    }
    P.blit(currentBuffer(), state.width, state.height, state.selection.pixels, state.selection.w, state.selection.h, state.selection.x, state.selection.y);
    state.selection = null;
  }

  function liftSelection() {
    const sel = state.selection;
    if (!sel || sel.floating) return;
    const pixels = new Uint8ClampedArray(sel.w * sel.h * 4);
    const buf = currentBuffer();
    for (let y = 0; y < sel.h; y++) {
      for (let x = 0; x < sel.w; x++) {
        const sx = sel.x + x;
        const sy = sel.y + y;
        if (sx < 0 || sy < 0 || sx >= state.width || sy >= state.height) continue;
        const c = P.get(buf, sx, sy, state.width);
        P.set(pixels, x, y, sel.w, c);
        P.set(buf, sx, sy, state.width, TRANSPARENT);
      }
    }
    sel.pixels = pixels;
    sel.floating = true;
  }

  function selectionAt(x, y) {
    const sel = state.selection;
    if (!sel) return false;
    return x >= sel.x && y >= sel.y && x < sel.x + sel.w && y < sel.y + sel.h;
  }

  function copySelection() {
    const sel = state.selection;
    if (!sel) return;
    const pixels = sel.floating ? P.clone(sel.pixels) : new Uint8ClampedArray(sel.w * sel.h * 4);
    if (!sel.floating) {
      const buf = currentBuffer();
      for (let y = 0; y < sel.h; y++) {
        for (let x = 0; x < sel.w; x++) {
          const sx = sel.x + x;
          const sy = sel.y + y;
          if (sx < 0 || sy < 0 || sx >= state.width || sy >= state.height) continue;
          P.set(pixels, x, y, sel.w, P.get(buf, sx, sy, state.width));
        }
      }
    }
    state.clipboard = { w: sel.w, h: sel.h, pixels };
  }

function eraseSelectionPixels() {
  const sel = state.selection;
  const buf = currentBuffer();
  for (let y = 0; y < sel.h; y++) {
    for (let x = 0; x < sel.w; x++) {
      const sx = sel.x + x;
      const sy = sel.y + y;
      if (sx < 0 || sy < 0 || sx >= state.width || sy >= state.height) continue;
      P.set(buf, sx, sy, state.width, TRANSPARENT);
    }
  }
}

function cutSelection() {
  if (!state.selection) return;
  pushUndo();
  copySelection();
  if (!state.selection.floating) eraseSelectionPixels();
  state.selection = null;
  render();
}

function deleteSelection() {
  if (!state.selection) return;
  if (!state.selection.floating) {
    pushUndo();
    eraseSelectionPixels();
  }
  state.selection = null;
  render();
}

  function pasteSelection() {
    if (!state.clipboard) return;
    pushUndo();
    commitSelection();
    state.selection = {
      x: 0,
      y: 0,
      w: state.clipboard.w,
      h: state.clipboard.h,
      floating: true,
      pixels: P.clone(state.clipboard.pixels),
    };
    setTool("select");
    render();
  }

  function nudgeSelection(dx, dy) {
    if (!state.selection) return;
    if (!state.selection.floating) {
      pushUndo();
      liftSelection();
    }
    state.selection.x += dx;
    state.selection.y += dy;
    markDirty();
    render();
  }

  let traceSample = null;

  function tracePixels() {
    const image = state.trace.image;
    if (!image) return null;
    if (traceSample && traceSample.image === image) return traceSample.data;
    const sample = document.createElement("canvas");
    sample.width = image.width;
    sample.height = image.height;
    const context = sample.getContext("2d", { willReadFrequently: true });
    context.drawImage(image, 0, 0);
    const data = context.getImageData(0, 0, sample.width, sample.height).data;
    traceSample = { image, data };
    return data;
  }

  function traceColorAt(spriteX, spriteY) {
    const image = state.trace.image;
    if (!state.trace.visible || state.trace.opacity <= 0 || !image || !state.trace.scale) return null;
    const px = Math.floor((spriteX - state.trace.x) / state.trace.scale);
    const py = Math.floor((spriteY - state.trace.y) / state.trace.scale);
    if (px < 0 || py < 0 || px >= image.width || py >= image.height) return null;
    const data = tracePixels();
    const i = (py * image.width + px) * 4;
    if (!data || data[i + 3] === 0) return null;
    return [data[i], data[i + 1], data[i + 2], 255];
  }

  function pickColor(x, y, button, exact) {
    const fromTrace = exact && !exact.spriteOnly ? traceColorAt(exact.x, exact.y) : null;
    let color = fromTrace;
    if (!color) {
      if (x < 0 || y < 0 || x >= state.width || y >= state.height) return;
      color = P.get(P.composite(state.layers, state.frame, state.width, state.height, false, state.layer), x, y, state.width);
    }
    if (button === 2) state.secondary = color;
    else state.primary = color;
    $("st-msg").textContent = fromTrace ? "Sampled the trace" : "Sampled the sprite";
    paintSwatches();
  }

  function spritePointFromEvent(evt) {
    const rect = canvas.getBoundingClientRect();
    return [
      (evt.clientX - rect.left - state.panX) / state.zoom,
      (evt.clientY - rect.top - state.panY) / state.zoom,
    ];
  }

  function cellFromEvent(evt) {
    const [x, y] = spritePointFromEvent(evt);
    return [Math.floor(x), Math.floor(y)];
  }

  function inside(x, y) {
    return x >= 0 && y >= 0 && x < state.width && y < state.height;
  }

  function onPointerDown(evt) {
    if (evt.button === 1 || (hand && evt.button === 0)) {
      pan = { x: evt.clientX, y: evt.clientY, panX: state.panX, panY: state.panY };
      try { canvas.setPointerCapture(evt.pointerId); } catch (err) { /* Synthetic events in tests have no capture. */ }
      return;
    }
    if (evt.button !== 0 && evt.button !== 2) return;
    const [x, y] = cellFromEvent(evt);
    hover = [x, y];
    if ((evt.altKey && state.tool !== "move") || state.tool === "picker") {
      const point = spritePointFromEvent(evt);
      pickColor(x, y, evt.button, { x: point[0], y: point[1], spriteOnly: evt.shiftKey });
      return;
    }
    if (state.traceMove) {
      pushUndo();
      gesture = { kind: "trace", x: evt.clientX, y: evt.clientY, ox: state.trace.x, oy: state.trace.y };
      try { canvas.setPointerCapture(evt.pointerId); } catch (err) { /* Synthetic events in tests have no capture. */ }
      return;
    }
    if (state.tool === "select") {
      if (state.selection && selectionAt(x, y)) {
        pushUndo();
        liftSelection();
        gesture = { kind: "select-move", x, y, ox: state.selection.x, oy: state.selection.y };
      } else {
        if (state.selection && state.selection.floating) {
          pushUndo();
          commitSelection();
        }
        gesture = { kind: "select", x0: x, y0: y, x1: x, y1: y };
      }
      try { canvas.setPointerCapture(evt.pointerId); } catch (err) { /* Synthetic events in tests have no capture. */ }
      return;
    }
    if (state.selection && state.selection.floating) commitSelection();
    else state.selection = null;
    if (!inside(x, y) && state.tool !== "line" && state.tool !== "rect" && state.tool !== "ellipse" && state.tool !== "move") return;
    pushUndo();
    const bufs = state.tool === "move" || state.tool === "magic" ? targets(evt) : [currentBuffer()];
    gesture = {
      kind: "draw",
      button: evt.button,
      x0: x,
      y0: y,
      last: [x, y],
      bufs,
      saved: bufs.map((buf) => ({ buf, data: P.clone(buf) })),
      once: new Set(),
    };
    applyTool(evt, x, y, true);
    try { canvas.setPointerCapture(evt.pointerId); } catch (err) { /* Synthetic events in tests have no capture. */ }
    render();
  }

  function onPointerMove(evt) {
    const [x, y] = cellFromEvent(evt);
    hover = [x, y];
    if (pan) {
      state.panX = pan.panX + (evt.clientX - pan.x);
      state.panY = pan.panY + (evt.clientY - pan.y);
      clampPan();
      render();
      return;
    }
    if (!gesture) {
      render();
      return;
    }
    if (gesture.kind === "trace") {
      state.trace.x = gesture.ox + (evt.clientX - gesture.x) / state.zoom;
      state.trace.y = gesture.oy + (evt.clientY - gesture.y) / state.zoom;
      render();
      return;
    }
    if (gesture.kind === "select") {
      gesture.x1 = x;
      gesture.y1 = y;
      render();
      return;
    }
    if (gesture.kind === "select-move") {
      state.selection.x = gesture.ox + (x - gesture.x);
      state.selection.y = gesture.oy + (y - gesture.y);
      render();
      return;
    }
    applyTool(evt, x, y, false);
    render();
  }

  function onPointerUp(evt) {
    if (pan) {
      pan = null;
      return;
    }
    if (!gesture) return;
    if (gesture.kind === "select") {
      const x0 = Math.min(gesture.x0, gesture.x1);
      const y0 = Math.min(gesture.y0, gesture.y1);
      const x1 = Math.max(gesture.x0, gesture.x1);
      const y1 = Math.max(gesture.y0, gesture.y1);
      const w = Math.min(state.width, x1 + 1) - Math.max(0, x0);
      const h = Math.min(state.height, y1 + 1) - Math.max(0, y0);
      if (w > 0 && h > 0) {
        state.selection = {
          x: Math.max(0, x0),
          y: Math.max(0, y0),
          w,
          h,
          floating: false,
          pixels: new Uint8ClampedArray(Math.max(1, w) * Math.max(1, h) * 4),
        };
      }
    }
    gesture = null;
    pan = null;
    render();
  }

  function maxZoom() {
    return Math.max(1, Math.floor(Math.min(view.clientWidth, view.clientHeight) / 10));
  }

  function clampPan() {
    const vw = Math.max(1, view.clientWidth);
    const vh = Math.max(1, view.clientHeight);
    const sw = state.width * state.zoom;
    const sh = state.height * state.zoom;
    const keep = 64;
    state.panX = Math.min(vw - keep, Math.max(keep - sw, state.panX));
    state.panY = Math.min(vh - keep, Math.max(keep - sh, state.panY));
  }

  function fit() {
    const vw = Math.max(1, view.clientWidth);
    const vh = Math.max(1, view.clientHeight);
    state.zoom = Math.max(1, Math.min(maxZoom(), Math.floor(Math.min(vw / state.width, vh / state.height))));
    state.panX = Math.round((vw - state.width * state.zoom) / 2);
    state.panY = Math.round((vh - state.height * state.zoom) / 2);
    clampPan();
  }

  function zoomAt(clientX, clientY, next) {
    const rect = canvas.getBoundingClientRect();
    const x = clientX - rect.left;
    const y = clientY - rect.top;
    const sx = (x - state.panX) / state.zoom;
    const sy = (y - state.panY) / state.zoom;
    state.zoom = Math.max(1, Math.min(maxZoom(), next));
    state.panX = x - sx * state.zoom;
    state.panY = y - sy * state.zoom;
    clampPan();
    render();
  }

  function drawChecker(x, y, w, h) {
    const cell = Math.max(state.zoom, 1);
    ctx.save();
    ctx.beginPath();
    ctx.rect(x, y, w, h);
    ctx.clip();
    ctx.fillStyle = "#1a1a1a";
    ctx.fillRect(x, y, w, h);
    ctx.fillStyle = "#2a2a2a";
    for (let py = 0; py < state.height; py++) {
      for (let px = 0; px < state.width; px++) {
        if (((px + py) & 1) === 0) ctx.fillRect(x + px * cell, y + py * cell, cell, cell);
      }
    }
    ctx.restore();
  }

  function blitPixels(pixels, w, h, dx, dy, alpha) {
    const off = document.createElement("canvas");
    off.width = w;
    off.height = h;
    const octx = off.getContext("2d");
    octx.putImageData(new ImageData(new Uint8ClampedArray(pixels), w, h), 0, 0);
    ctx.save();
    ctx.imageSmoothingEnabled = false;
    ctx.globalAlpha = alpha;
    ctx.drawImage(off, dx, dy, w * state.zoom, h * state.zoom);
    ctx.restore();
  }

  function spritePixels(frame, dim) {
    const pixels = P.composite(state.layers, frame, state.width, state.height, dim, state.layer);
    const sel = state.selection;
    if (sel && sel.floating && frame === state.frame) {
      P.blit(pixels, state.width, state.height, sel.pixels, sel.w, sel.h, sel.x, sel.y);
    }
    return pixels;
  }

  function render() {
    syncTraceCount();
    const dpr = window.devicePixelRatio || 1;
    const cssW = view.clientWidth;
    const cssH = view.clientHeight;
    if (canvas.width !== Math.floor(cssW * dpr) || canvas.height !== Math.floor(cssH * dpr)) {
      canvas.width = Math.max(1, Math.floor(cssW * dpr));
      canvas.height = Math.max(1, Math.floor(cssH * dpr));
    }
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, cssW, cssH);
    ctx.fillStyle = "#101010";
    ctx.fillRect(0, 0, cssW, cssH);
    const ox = state.panX;
    const oy = state.panY;
    if (state.tile) {
      for (let ty = -1; ty <= 1; ty++) {
        for (let tx = -1; tx <= 1; tx++) {
          if (tx === 0 && ty === 0) continue;
          blitPixels(spritePixels(state.frame, false), state.width, state.height, ox + tx * state.width * state.zoom, oy + ty * state.height * state.zoom, 1);
        }
      }
    }
    drawChecker(ox, oy, state.width * state.zoom, state.height * state.zoom);
    if (state.onion && state.frame > 0) {
      blitPixels(spritePixels(state.frame - 1, false), state.width, state.height, ox, oy, 0.35);
    }
    blitPixels(spritePixels(state.frame, state.dim), state.width, state.height, ox, oy, 1);
    if (state.trace.visible && state.trace.image) {
      const img = state.trace.image;
      ctx.save();
      ctx.globalAlpha = state.trace.opacity;
      ctx.imageSmoothingEnabled = false;
      ctx.drawImage(img, ox + state.trace.x * state.zoom, oy + state.trace.y * state.zoom, img.width * state.trace.scale * state.zoom, img.height * state.trace.scale * state.zoom);
      ctx.restore();
    }
    if (state.grid && state.zoom >= 6) {
      ctx.save();
      ctx.beginPath();
      ctx.rect(ox, oy, state.width * state.zoom, state.height * state.zoom);
      ctx.clip();
      ctx.strokeStyle = "rgba(255,255,255,0.16)";
      ctx.lineWidth = 1;
      ctx.beginPath();
      for (let x = 0; x <= state.width; x++) {
        const px = Math.round(ox + x * state.zoom) + 0.5;
        ctx.moveTo(px, oy);
        ctx.lineTo(px, oy + state.height * state.zoom);
      }
      for (let y = 0; y <= state.height; y++) {
        const py = Math.round(oy + y * state.zoom) + 0.5;
        ctx.moveTo(ox, py);
        ctx.lineTo(ox + state.width * state.zoom, py);
      }
      ctx.stroke();
      ctx.restore();
    }
    if (hover && state.tool !== "picker" && state.tool !== "select" && state.tool !== "move" && !state.traceMove) {
      const [hx, hy] = hover;
      const o = P.stampOrigin(state.size);
      ctx.save();
      ctx.strokeStyle = "rgba(255, 220, 170, 0.9)";
      ctx.lineWidth = 1;
      ctx.strokeRect(ox + (hx + o) * state.zoom + 0.5, oy + (hy + o) * state.zoom + 0.5, state.size * state.zoom - 1, state.size * state.zoom - 1);
      ctx.restore();
    }
    const sel = gesture && gesture.kind === "select"
      ? {
          x: Math.min(gesture.x0, gesture.x1),
          y: Math.min(gesture.y0, gesture.y1),
          w: Math.abs(gesture.x1 - gesture.x0) + 1,
          h: Math.abs(gesture.y1 - gesture.y0) + 1,
        }
      : state.selection;
    if (sel) {
      ctx.save();
      ctx.strokeStyle = "#ffb15a";
      ctx.setLineDash([3, 3]);
      ctx.lineDashOffset = -((performance.now() / 80) % 12);
      ctx.strokeRect(ox + sel.x * state.zoom + 0.5, oy + sel.y * state.zoom + 0.5, sel.w * state.zoom - 1, sel.h * state.zoom - 1);
      ctx.restore();
    }
    paintStatus();
    paintThumbs();
    paintPreview();
  }

  function paintStatus() {
    $("st-size").textContent = state.width + " × " + state.height;
    $("st-zoom").textContent = "zoom " + state.zoom + "×";
    $("st-frame").textContent = "frame " + (state.frame + 1) + " / " + frameCount();
    $("zoom-label").textContent = state.zoom + "×";
    if (!hover) {
      $("st-cell").textContent = "—";
      return;
    }
    const [x, y] = hover;
    let text = inside(x, y) ? x + ", " + y : "outside";
    if (gesture && gesture.kind === "draw" && (state.tool === "rect" || state.tool === "ellipse" || state.tool === "line")) {
      const x0 = gesture.x0;
      const y0 = gesture.y0;
      const x1 = gesture.box ? gesture.box[2] : gesture.x1 || x;
      const y1 = gesture.box ? gesture.box[3] : gesture.y1 || y;
      text = x0 + ", " + y0 + "  " + (Math.abs(x1 - x0) + 1) + " × " + (Math.abs(y1 - y0) + 1);
    }
    if (gesture && gesture.kind === "draw" && state.tool === "move" && gesture.shift) {
      text = "move " + gesture.shift[0] + ", " + gesture.shift[1];
    }
    $("st-cell").textContent = text;
  }

  function paintThumbs() {
    document.querySelectorAll(".frame canvas").forEach((thumb, index) => {
      const g = thumb.getContext("2d");
      const pixels = P.composite(state.layers, index, state.width, state.height, false, state.layer);
      const off = document.createElement("canvas");
      off.width = state.width;
      off.height = state.height;
      off.getContext("2d").putImageData(new ImageData(new Uint8ClampedArray(pixels), state.width, state.height), 0, 0);
      g.imageSmoothingEnabled = false;
      g.clearRect(0, 0, thumb.width, thumb.height);
      g.fillStyle = "#111";
      g.fillRect(0, 0, thumb.width, thumb.height);
      const scale = Math.min(thumb.width / state.width, thumb.height / state.height);
      const dw = state.width * scale;
      const dh = state.height * scale;
      g.drawImage(off, (thumb.width - dw) / 2, (thumb.height - dh) / 2, dw, dh);
    });
  }

function paintPreview() {
  const frames = frameCount();
  const frame = state.playing ? previewTick % frames : state.frame;
    const pixels = P.composite(state.layers, frame, state.width, state.height, false, state.layer);
    const off = document.createElement("canvas");
    off.width = state.width;
    off.height = state.height;
    off.getContext("2d").putImageData(new ImageData(new Uint8ClampedArray(pixels), state.width, state.height), 0, 0);
    pctx.imageSmoothingEnabled = false;
    pctx.fillStyle = "#101010";
    pctx.fillRect(0, 0, preview.width, preview.height);
    const scale = Math.min(preview.width / state.width, preview.height / state.height);
    const dw = state.width * scale;
    const dh = state.height * scale;
    pctx.drawImage(off, (preview.width - dw) / 2, (preview.height - dh) / 2, dw, dh);
  }

  function paintSwatches() {
    const fill = (el, color) => {
      if (color[3] === 0) {
        el.style.background = "";
        el.classList.add("checker");
      } else {
        el.classList.remove("checker");
        el.style.background = "rgb(" + color[0] + "," + color[1] + "," + color[2] + ")";
      }
    };
    fill($("swatch-primary"), state.primary);
    fill($("swatch-secondary"), state.secondary);
    const hex = state.primary[3] === 0 ? "#000000" : "#" + [0, 1, 2].map((i) => state.primary[i].toString(16).padStart(2, "0")).join("");
    $("color-input").value = hex;
  }

  function paintChrome() {
    document.querySelectorAll("[data-tool]").forEach((button) => button.classList.toggle("on", button.dataset.tool === state.tool));
    document.querySelectorAll("[data-size]").forEach((button) => button.classList.toggle("on", Number(button.dataset.size) === state.size));
    $("btn-grid").classList.toggle("on", state.grid);
    $("btn-tile").classList.toggle("on", state.tile);
    $("btn-onion").classList.toggle("on", state.onion);
    $("btn-dim").classList.toggle("on", state.dim);
    $("trace-move").classList.toggle("on", state.traceMove);
    $("trace-visible").classList.toggle("on", state.trace.visible);
    $("trace-scale").disabled = !state.traceMove;
    $("trace-scale-num").disabled = !state.traceMove;
    $("trace-scale").title = state.traceMove
      ? "100% fits the picture inside the sprite. Drag left to make it smaller."
      : "Turn on Move trace to change the size.";
    $("btn-play").textContent = state.playing ? "Pause" : "Play";
    $("btn-play").classList.toggle("on", state.playing);
    paintSwatches();
    view.style.cursor = hand ? "grab" : state.tool === "picker" ? "crosshair" : state.traceMove ? "move" : state.tool === "move" || state.tool === "select" ? "default" : "crosshair";
  }

  function setTool(name) {
    state.tool = name;
    if (name !== "select" && state.selection && state.selection.floating) {
      commitSelection();
      render();
    }
    paintChrome();
  }

  function setSize(size) {
    state.size = Math.max(1, Math.min(32, size));
    paintChrome();
    render();
  }

  function buildFrames() {
    const box = $("frames");
    box.innerHTML = "";
    for (let i = 0; i < frameCount(); i++) {
      const button = document.createElement("button");
      button.type = "button";
      button.className = "frame" + (i === state.frame ? " on" : "");
      const thumb = document.createElement("canvas");
      thumb.width = 64;
      thumb.height = 52;
      const label = document.createElement("span");
      label.textContent = state.traces[i] && state.traces[i].dataUrl ? String(i + 1) + "·" : String(i + 1);
      button.title = state.traces[i] && state.traces[i].dataUrl ? "Frame " + (i + 1) + " has its own tracing picture" : "Frame " + (i + 1);
      button.append(thumb, label);
      button.addEventListener("click", () => {
        syncTraceCount();
        state.frame = i;
        state.trace = state.traces[state.frame];
        traceSample = null;
        syncTraceControls();
        buildFrames();
        render();
      });
      box.append(button);
    }
    $("frame-del").disabled = frameCount() === 1;
  }

  function buildLayers() {
    const box = $("layers");
    box.innerHTML = "";
    for (let i = state.layers.length - 1; i >= 0; i--) {
      const layer = state.layers[i];
      const row = document.createElement("div");
      row.className = "layer" + (i === state.layer ? " on" : "");
      const eye = document.createElement("button");
      eye.type = "button";
      eye.className = "eye";
      eye.textContent = layer.visible ? "●" : "○";
      eye.title = "Show or hide";
      eye.addEventListener("click", (evt) => {
        evt.stopPropagation();
        layer.visible = !layer.visible;
        buildLayers();
        render();
      });
      const name = document.createElement("div");
      name.className = "name";
      name.textContent = layer.name;
      name.title = "Double-click to rename";
      name.addEventListener("dblclick", () => {
        const next = prompt("Layer name", layer.name);
        if (next && next.trim()) {
          layer.name = next.trim();
          buildLayers();
        }
      });
      const ops = document.createElement("div");
      ops.className = "ops";
      const up = document.createElement("button");
      up.type = "button";
      up.textContent = "↑";
      up.title = "Move up. Shift sends it to the top.";
      up.addEventListener("click", (evt) => {
        evt.stopPropagation();
        moveLayer(i, evt.shiftKey ? state.layers.length - 1 : Math.min(state.layers.length - 1, i + 1));
      });
      const down = document.createElement("button");
      down.type = "button";
      down.textContent = "↓";
      down.title = "Move down. Shift sends it to the bottom.";
      down.addEventListener("click", (evt) => {
        evt.stopPropagation();
        moveLayer(i, evt.shiftKey ? 0 : Math.max(0, i - 1));
      });
      const merge = document.createElement("button");
      merge.type = "button";
      merge.textContent = "⤵";
      merge.title = "Merge down";
      merge.disabled = i === 0;
      merge.addEventListener("click", (evt) => {
        evt.stopPropagation();
        mergeDown(i);
      });
      const del = document.createElement("button");
      del.type = "button";
      del.textContent = "✕";
      del.disabled = state.layers.length === 1;
      del.addEventListener("click", (evt) => {
        evt.stopPropagation();
        pushUndo();
        state.layers.splice(i, 1);
        state.layer = Math.min(state.layer, state.layers.length - 1);
        buildLayers();
        render();
      });
      ops.append(up, down, merge, del);
      const opacity = document.createElement("input");
      opacity.type = "range";
      opacity.min = "0";
      opacity.max = "100";
      opacity.value = String(Math.round(layer.opacity * 100));
      opacity.addEventListener("click", (evt) => evt.stopPropagation());
      opacity.addEventListener("input", () => {
        layer.opacity = Number(opacity.value) / 100;
        render();
      });
      row.append(eye, name, ops, opacity);
      row.addEventListener("click", () => {
        state.layer = i;
        buildLayers();
        render();
      });
      box.append(row);
    }
  }

  function moveLayer(from, to) {
    if (from === to) return;
    pushUndo();
    const [layer] = state.layers.splice(from, 1);
    state.layers.splice(to, 0, layer);
    state.layer = to;
    buildLayers();
    render();
  }

  function mergeDown(index) {
    if (index <= 0) return;
    pushUndo();
    const upper = state.layers[index];
    const lower = state.layers[index - 1];
    lower.frames = lower.frames.map((frame, fi) => P.composite([
      { name: lower.name, opacity: lower.opacity, visible: true, frames: [frame] },
      { name: upper.name, opacity: upper.opacity, visible: true, frames: [upper.frames[fi]] },
    ], 0, state.width, state.height, false, 1));
    lower.opacity = 1;
    state.layers.splice(index, 1);
    state.layer = index - 1;
    buildLayers();
    render();
  }

  function buildPalette() {
    const box = $("palette");
    box.innerHTML = "";
    state.palette.forEach((color) => {
      const chip = document.createElement("button");
      chip.type = "button";
      chip.className = "chip" + (color[3] === 0 ? " checker" : "");
      if (color[3] !== 0) chip.style.background = "rgb(" + color[0] + "," + color[1] + "," + color[2] + ")";
      chip.title = color[3] === 0 ? "Transparent" : color.join(",");
      chip.addEventListener("click", () => {
        state.primary = color.slice();
        paintSwatches();
      });
      chip.addEventListener("contextmenu", (evt) => {
        evt.preventDefault();
        state.secondary = color.slice();
        paintSwatches();
      });
      box.append(chip);
    });
  }

  function loadTraceInto(trace, dataUrl, fitImage) {
    const showing = () => trace === state.trace;
    return new Promise((resolve) => {
      if (!dataUrl) {
        const hadImage = Boolean(trace.dataUrl);
        if (showing()) traceSample = null;
        trace.dataUrl = null;
        trace.image = null;
        if (hadImage) markDirty();
        if (showing()) {
          syncTraceControls();
          render();
        }
        resolve();
        return;
      }
      const image = new Image();
      image.onload = () => {
        if (showing()) traceSample = null;
        trace.image = image;
        trace.dataUrl = dataUrl;
        if (fitImage) {
          trace.scale = window.TraceScale.fitScale(state.width, state.height, image.width, image.height);
          trace.x = 0;
          trace.y = 0;
          trace.visible = true;
          trace.opacity = trace.opacity || 0.45;
          markDirty();
        }
        if (showing()) {
          syncTraceControls();
          render();
        }
        resolve();
      };
      image.onerror = () => resolve();
      image.src = dataUrl;
    });
  }

  function loadTrace(dataUrl, fitImage) {
    syncTraceCount();
    return loadTraceInto(state.trace, dataUrl, fitImage);
  }

  function traceFit() {
    const image = state.trace.image;
    if (!image) return 1;
    return window.TraceScale.fitScale(state.width, state.height, image.width, image.height);
  }

  function syncTraceControls() {
    $("trace-opacity").value = String(Math.round(state.trace.opacity * 100));
    const pct = (state.trace.scale / traceFit()) * 100;
    $("trace-scale").value = String(window.TraceScale.sliderFromPercent(pct));
    $("trace-scale-num").value = String(Math.round(pct * 10) / 10);
    paintChrome();
  }

  function setTracePercent(pct, source) {
    if (!state.traceMove) return;
    const clamped = Math.min(window.TraceScale.MAX, Math.max(window.TraceScale.MIN, pct));
    state.trace.scale = traceFit() * (clamped / 100);
    markDirty();
    if (source !== "slider") $("trace-scale").value = String(window.TraceScale.sliderFromPercent(clamped));
    if (source !== "number") $("trace-scale-num").value = String(Math.round(clamped * 10) / 10);
    render();
  }

  function bytesToB64(bytes) {
    let text = "";
    const chunk = 0x8000;
    for (let i = 0; i < bytes.length; i += chunk) {
      text += String.fromCharCode.apply(null, bytes.subarray(i, i + chunk));
    }
    return btoa(text);
  }

  function b64ToBytes(text) {
    const raw = atob(text);
    const out = new Uint8ClampedArray(raw.length);
    for (let i = 0; i < raw.length; i++) out[i] = raw.charCodeAt(i);
    return out;
  }

  function serialize() {
    return JSON.stringify({
      version: 2,
      name: state.name,
      width: state.width,
      height: state.height,
      fps: state.fps,
      layers: state.layers.map((layer) => ({
        name: layer.name,
        opacity: layer.opacity,
        visible: layer.visible,
        frames: layer.frames.map((frame) => bytesToB64(frame)),
      })),
      traces: state.traces.map((trace) => ({
        dataUrl: trace.dataUrl,
        opacity: trace.opacity,
        visible: trace.visible,
        scale: trace.scale,
        x: trace.x,
        y: trace.y,
      })),
    });
  }

  async function openText(text, path) {
    const data = JSON.parse(text);
    if (!data || (data.version !== 1 && data.version !== 2) || !Array.isArray(data.layers)) throw new Error("This file is not a sprite project.");
    state.width = data.width;
    state.height = data.height;
    state.name = data.name || "sprite";
    state.fps = data.fps || 8;
    state.layers = data.layers.map((layer) => ({
      name: layer.name || "Layer",
      opacity: layer.opacity == null ? 1 : layer.opacity,
      visible: layer.visible !== false,
      frames: layer.frames.map((frame) => b64ToBytes(frame)),
    }));
    state.frame = 0;
    state.layer = state.layers.length - 1;
    state.selection = null;
    state.undo = [];
    state.redo = [];
    state.dirty = false;
    state.path = path || null;
    $("doc-name").textContent = state.name;
    $("doc-name").classList.remove("dirty");
    $("fps").value = state.fps;
    const count = state.layers[0].frames.length;
    const records = Array.isArray(data.traces)
      ? data.traces.slice(0, count)
      : Array.from({ length: count }, () => data.trace || {});
    while (records.length < count) records.push({});
    state.traces = records.map((trace) => ({
      dataUrl: trace.dataUrl || null,
      image: null,
      opacity: trace.opacity == null ? 0.45 : trace.opacity,
      visible: trace.visible !== false,
      scale: trace.scale == null ? 1 : trace.scale,
      x: trace.x || 0,
      y: trace.y || 0,
    }));
    state.frame = 0;
    state.trace = state.traces[0];
    await Promise.all(state.traces.map((trace) => (trace.dataUrl ? loadTraceInto(trace, trace.dataUrl, false) : null)));
    buildLayers();
    buildFrames();
    buildPalette();
    fit();
    render();
  }

  function download(filename, url) {
    const link = document.createElement("a");
    link.href = url;
    link.download = filename;
    link.click();
  }

  async function saveProject() {
    const data = serialize();
    const suggested = state.name + ".sprite.json";
    if (window.desk) {
      const saved = await window.desk.saveProject(state.path || suggested, data);
      if (!saved) return null;
      state.path = saved;
      state.name = saved.split(/[\\/]/).pop().replace(/\.sprite\.json$/i, "").replace(/\.json$/i, "");
    } else {
      download(suggested, URL.createObjectURL(new Blob([data], { type: "application/json" })));
    }
    state.dirty = false;
    $("doc-name").textContent = state.name;
    $("doc-name").classList.remove("dirty");
    $("st-msg").textContent = "Saved";
    return state.path || suggested;
  }

  async function openProject() {
    if (state.dirty && !confirm("Open another sprite? Unsaved drawing will be dropped.")) return;
    if (window.desk) {
      const opened = await window.desk.openProject();
      if (!opened) return;
      await openText(opened.text, opened.path);
      return;
    }
    $("file-project").click();
  }

  function exportCanvas(width, height, draw) {
    const out = document.createElement("canvas");
    out.width = width;
    out.height = height;
    draw(out.getContext("2d"), out);
    return out.toDataURL("image/png");
  }

  function putFrame(context, pixels, dx) {
    const off = document.createElement("canvas");
    off.width = state.width;
    off.height = state.height;
    off.getContext("2d").putImageData(new ImageData(new Uint8ClampedArray(pixels), state.width, state.height), 0, 0);
    context.drawImage(off, dx, 0);
  }

  async function exportAs(kind) {
    let dataUrl;
    let filename;
    let fileKind = "png";
    if (kind === "png") {
      dataUrl = exportCanvas(state.width, state.height, (context) => {
        putFrame(context, P.composite(state.layers, state.frame, state.width, state.height, false, state.layer), 0);
      });
      filename = state.name + "-frame-" + (state.frame + 1) + ".png";
    } else if (kind === "sheet") {
      const count = frameCount();
      dataUrl = exportCanvas(state.width * count, state.height, (context) => {
        for (let i = 0; i < count; i++) {
          putFrame(context, P.composite(state.layers, i, state.width, state.height, false, state.layer), i * state.width);
        }
      });
      filename = state.name + "-sheet.png";
    } else {
      const frames = [];
      for (let i = 0; i < frameCount(); i++) frames.push(P.composite(state.layers, i, state.width, state.height, false, state.layer));
      const gif = window.Gif.encode(frames, state.width, state.height, state.fps);
      dataUrl = "data:image/gif;base64," + bytesToB64(gif);
      filename = state.name + ".gif";
      fileKind = "gif";
    }
    if (window.desk) {
      const saved = await window.desk.exportFile(filename, dataUrl, fileKind);
      if (saved) $("st-msg").textContent = "Exported " + saved.split(/[\\/]/).pop();
    } else {
      download(filename, dataUrl);
      $("st-msg").textContent = "Exported " + filename;
    }
  }

  function readFile(file) {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = () => resolve(reader.result);
      reader.onerror = () => reject(reader.error);
      if (file.name && file.name.endsWith(".json")) reader.readAsText(file);
      else reader.readAsDataURL(file);
    });
  }

  async function importImage(src) {
    const image = await loadImage(src);
    pushUndo();
    if (image.height === state.height && image.width >= state.width && image.width % state.width === 0) {
      const count = image.width / state.width;
      const sample = sampleImage(image);
      while (frameCount() < state.frame + count) state.layers.forEach((layer) => layer.frames.push(blank()));
      for (let f = 0; f < count; f++) {
        const buf = state.layers[state.layer].frames[state.frame + f];
        buf.fill(0);
        for (let y = 0; y < state.height; y++) {
          for (let x = 0; x < state.width; x++) {
            P.set(buf, x, y, state.width, pixelAt(sample, image.width, f * state.width + x, y));
          }
        }
      }
    } else {
      const buf = currentBuffer();
      const sample = sampleImage(image);
      buf.fill(0);
      for (let y = 0; y < state.height; y++) {
        for (let x = 0; x < state.width; x++) {
          const sx = Math.min(image.width - 1, Math.floor(((x + 0.5) * image.width) / state.width));
          const sy = Math.min(image.height - 1, Math.floor(((y + 0.5) * image.height) / state.height));
          P.set(buf, x, y, state.width, pixelAt(sample, image.width, sx, sy));
        }
      }
    }
    buildFrames();
    render();
  }

  function loadImage(src) {
    return new Promise((resolve, reject) => {
      const image = new Image();
      image.onload = () => resolve(image);
      image.onerror = reject;
      image.src = src;
    });
  }

  function sampleImage(image) {
    const c = document.createElement("canvas");
    c.width = image.width;
    c.height = image.height;
    const g = c.getContext("2d");
    g.drawImage(image, 0, 0);
    return g.getImageData(0, 0, image.width, image.height).data;
  }

  function pixelAt(data, w, x, y) {
    const i = (y * w + x) * 4;
    return [data[i], data[i + 1], data[i + 2], data[i + 3]];
  }

  function transform(kind) {
    pushUndo();
    if (kind === "rotate") {
      const w = state.width;
      const h = state.height;
      state.layers.forEach((layer) => {
        layer.frames = layer.frames.map((frame) => P.rotate(frame, w, h));
      });
      state.width = h;
      state.height = w;
      state.selection = null;
      fit();
      buildFrames();
      render();
      return;
    }
    const layer = state.layers[state.layer];
    layer.frames[state.frame] = P.flip(layer.frames[state.frame], state.width, state.height, kind === "flip-h");
    render();
  }

  function resizeTo(w, h) {
    pushUndo();
    state.layers.forEach((layer) => {
      layer.frames = layer.frames.map((frame) => {
        const next = new Uint8ClampedArray(w * h * 4);
        P.blit(next, w, h, frame, state.width, state.height, 0, 0);
        return next;
      });
    });
    state.width = w;
    state.height = h;
    state.selection = null;
    fit();
    buildFrames();
    render();
  }

  function onKey(evt) {
    const typing = evt.target && evt.target.matches && evt.target.matches("input, textarea");
    if (typing) return;
    if (evt.key === " " && !evt.ctrlKey && !evt.metaKey) {
      evt.preventDefault();
      if (!evt.repeat) {
        hand = true;
        paintChrome();
      }
      return;
    }
    const key = evt.key.toLowerCase();
    const ctrl = evt.ctrlKey || evt.metaKey;
    if (ctrl && key === "z" && !evt.shiftKey) {
      evt.preventDefault();
      undo();
      return;
    }
    if (ctrl && (key === "y" || (key === "z" && evt.shiftKey))) {
      evt.preventDefault();
      redo();
      return;
    }
    if (ctrl && key === "s") {
      evt.preventDefault();
      saveProject();
      return;
    }
    if (ctrl && key === "o") {
      evt.preventDefault();
      openProject();
      return;
    }
    if (ctrl && key === "c") {
      evt.preventDefault();
      copySelection();
      return;
    }
    if (ctrl && key === "x") {
      evt.preventDefault();
      cutSelection();
      return;
    }
    if (ctrl && key === "v") {
      evt.preventDefault();
      pasteSelection();
      return;
    }
    if (key === "escape") {
      if (state.selection && state.selection.floating) commitSelection();
      else state.selection = null;
      gesture = null;
      render();
      return;
    }
    if ((key === "delete" || key === "backspace") && state.selection) {
      evt.preventDefault();
      deleteSelection();
      return;
    }
    if (state.selection && key.startsWith("arrow") && !evt.shiftKey) {
      evt.preventDefault();
      const dir = { arrowleft: [-1, 0], arrowright: [1, 0], arrowup: [0, -1], arrowdown: [0, 1] }[key];
      nudgeSelection(dir[0], dir[1]);
      return;
    }
    if (evt.shiftKey && key.startsWith("arrow")) {
      const step = state.zoom;
      const dir = { arrowleft: [step, 0], arrowright: [-step, 0], arrowup: [0, step], arrowdown: [0, -step] }[key];
      state.panX += dir[0];
      state.panY += dir[1];
      clampPan();
      render();
      return;
    }
    if (key === "[") setSize(state.size - 1);
    if (key === "]") setSize(state.size + 1);
    if (key === "x" && !ctrl) {
      const swap = state.primary;
      state.primary = state.secondary;
      state.secondary = swap;
      paintSwatches();
    }
    if (evt.altKey && key === "g") {
      state.grid = !state.grid;
      paintChrome();
      render();
    }
    if (evt.altKey && key === "l") {
      state.dim = !state.dim;
      paintChrome();
      render();
    }
    if (key === "+" || key === "=") zoomAt(view.clientWidth / 2 + view.getBoundingClientRect().left, view.clientHeight / 2 + view.getBoundingClientRect().top, state.zoom + (ctrl ? 4 : 1));
    if (key === "-" || key === "_") zoomAt(view.clientWidth / 2 + view.getBoundingClientRect().left, view.clientHeight / 2 + view.getBoundingClientRect().top, state.zoom - (ctrl ? 4 : 1));
    if (key === "0") {
      fit();
      render();
    }
    const tools = { p: "pen", v: "mirror", b: "bucket", a: "magic", e: "eraser", l: "line", r: "rect", c: "ellipse", m: "move", u: "lighten", t: "dither", o: "picker", s: "select" };
    if (!ctrl && !evt.altKey && tools[key]) setTool(tools[key]);
  }

  function bind() {
    document.querySelectorAll("[data-tool]").forEach((button) => button.addEventListener("click", () => setTool(button.dataset.tool)));
    document.querySelectorAll("[data-size]").forEach((button) => button.addEventListener("click", () => setSize(Number(button.dataset.size))));
    canvas.addEventListener("pointerdown", onPointerDown);
    canvas.addEventListener("pointermove", onPointerMove);
    canvas.addEventListener("pointerup", onPointerUp);
    canvas.addEventListener("pointerleave", () => {
      if (!gesture && !pan) {
        hover = null;
        render();
      }
    });
    canvas.addEventListener("contextmenu", (evt) => evt.preventDefault());
    view.addEventListener("wheel", (evt) => {
      evt.preventDefault();
      const step = evt.ctrlKey ? 4 : 1;
      zoomAt(evt.clientX, evt.clientY, state.zoom + (evt.deltaY < 0 ? step : -step));
    }, { passive: false });
    window.addEventListener("keydown", onKey);
    window.addEventListener("keyup", (evt) => {
      if (evt.key !== " ") return;
      hand = false;
      paintChrome();
    });
    window.addEventListener("blur", () => {
      hand = false;
      paintChrome();
    });
    window.addEventListener("resize", () => {
      clampPan();
      render();
    });
    new ResizeObserver(() => {
      if (!canvas.width) {
        fit();
        render();
      }
    }).observe(view);

    $("btn-new").addEventListener("click", () => {
      if (state.dirty && !confirm("Start a new sprite? Unsaved drawing will be dropped.")) return;
      $("dlg-new").showModal();
    });
    $("dlg-new").addEventListener("close", () => {
      if ($("dlg-new").returnValue !== "ok") return;
      resetDocument(clampDim($("new-w").value), clampDim($("new-h").value), "sprite");
    });
    $("btn-resize").addEventListener("click", () => {
      $("resize-w").value = state.width;
      $("resize-h").value = state.height;
      $("dlg-resize").showModal();
    });
    $("dlg-resize").addEventListener("close", () => {
      if ($("dlg-resize").returnValue !== "ok") return;
      resizeTo(clampDim($("resize-w").value), clampDim($("resize-h").value));
    });
    $("btn-help").addEventListener("click", () => $("dlg-help").showModal());
    $("btn-undo").addEventListener("click", undo);
    $("btn-redo").addEventListener("click", redo);
    $("btn-save").addEventListener("click", saveProject);
    $("btn-open").addEventListener("click", openProject);
    $("btn-import").addEventListener("click", () => {
      if (window.desk) {
        window.desk.pickImage().then((picked) => picked && importImage(picked.dataUrl));
      } else $("file-image").click();
    });
    $("btn-export").addEventListener("click", () => $("btn-export").parentElement.classList.toggle("open"));
    document.querySelectorAll("[data-export]").forEach((button) => button.addEventListener("click", () => {
      $("btn-export").parentElement.classList.remove("open");
      exportAs(button.dataset.export);
    }));
    document.addEventListener("click", (evt) => {
      if (!evt.target.closest(".menu")) $("btn-export").parentElement.classList.remove("open");
    });
    $("btn-grid").addEventListener("click", () => { state.grid = !state.grid; paintChrome(); render(); });
    $("btn-tile").addEventListener("click", () => { state.tile = !state.tile; paintChrome(); render(); });
    $("btn-onion").addEventListener("click", () => { state.onion = !state.onion; paintChrome(); render(); });
    $("btn-dim").addEventListener("click", () => { state.dim = !state.dim; paintChrome(); render(); });
    const zoomCenter = (next) => {
      const rect = view.getBoundingClientRect();
      zoomAt(rect.left + rect.width / 2, rect.top + rect.height / 2, next);
    };
    $("zoom-in").addEventListener("click", () => zoomCenter(state.zoom + 1));
    $("zoom-out").addEventListener("click", () => zoomCenter(state.zoom - 1));
    $("flip-h").addEventListener("click", () => transform("flip-h"));
    $("flip-v").addEventListener("click", () => transform("flip-v"));
    $("rotate").addEventListener("click", () => transform("rotate"));
    $("zoom-fit").addEventListener("click", () => { fit(); render(); });
    $("frame-add").addEventListener("click", () => {
      pushUndo();
      syncTraceCount();
      state.layers.forEach((layer) => layer.frames.splice(state.frame + 1, 0, blank()));
      state.traces.splice(state.frame + 1, 0, blankTrace());
      state.frame += 1;
      state.trace = state.traces[state.frame];
      traceSample = null;
      buildFrames();
      syncTraceControls();
      render();
    });
    $("frame-dup").addEventListener("click", () => {
      pushUndo();
      syncTraceCount();
      state.layers.forEach((layer) => layer.frames.splice(state.frame + 1, 0, P.clone(layer.frames[state.frame])));
      state.traces.splice(state.frame + 1, 0, copyTrace(state.trace));
      state.frame += 1;
      state.trace = state.traces[state.frame];
      buildFrames();
      syncTraceControls();
      render();
    });
    $("frame-del").addEventListener("click", () => {
      if (frameCount() === 1) return;
      pushUndo();
      syncTraceCount();
      state.layers.forEach((layer) => layer.frames.splice(state.frame, 1));
      state.traces.splice(state.frame, 1);
      state.frame = Math.min(state.frame, frameCount() - 1);
      state.trace = state.traces[state.frame];
      traceSample = null;
      buildFrames();
      syncTraceControls();
      render();
    });
    $("frame-left").addEventListener("click", () => moveFrame(-1));
    $("frame-right").addEventListener("click", () => moveFrame(1));
    $("layer-add").addEventListener("click", (evt) => {
      pushUndo();
      const layer = evt.shiftKey
        ? makeLayer(state.layers[state.layer].name + " copy", state.layers[state.layer].frames.map((frame) => P.clone(frame)))
        : makeLayer("Layer " + (state.layers.length + 1));
      state.layers.push(layer);
      state.layer = state.layers.length - 1;
      buildLayers();
      render();
    });
    $("palette-add").addEventListener("click", () => {
      if (!state.palette.some((color) => P.same(color, state.primary))) state.palette.push(state.primary.slice());
      buildPalette();
    });
    const openColor = (which) => {
      colorTarget = which;
      const color = which === "primary" ? state.primary : state.secondary;
      if (color[3] !== 0) {
        $("color-input").value = "#" + [0, 1, 2].map((i) => color[i].toString(16).padStart(2, "0")).join("");
      }
      $("color-input").click();
    };
    $("swatch-primary").addEventListener("click", () => openColor("primary"));
    $("swatch-secondary").addEventListener("click", () => openColor("secondary"));
    $("btn-swap").addEventListener("click", () => {
      const swap = state.primary;
      state.primary = state.secondary;
      state.secondary = swap;
      paintSwatches();
    });
    $("color-input").addEventListener("input", () => {
      const hex = $("color-input").value;
      const n = parseInt(hex.slice(1), 16);
      const color = [(n >> 16) & 255, (n >> 8) & 255, n & 255, 255];
      if (colorTarget === "secondary") state.secondary = color;
      else state.primary = color;
      paintSwatches();
    });
    $("trace-pick").addEventListener("click", () => {
      if (window.desk) window.desk.pickImage().then((picked) => picked && loadTrace(picked.dataUrl, true));
      else $("file-trace").click();
    });
    $("trace-clear").addEventListener("click", () => loadTrace(null, false));
    $("trace-opacity").addEventListener("input", () => {
      state.trace.opacity = Number($("trace-opacity").value) / 100;
      render();
    });
    $("trace-scale").addEventListener("input", () => {
      setTracePercent(window.TraceScale.percentFromSlider($("trace-scale").value), "slider");
    });
    $("trace-scale-num").addEventListener("change", () => {
      const pct = Number($("trace-scale-num").value);
      if (!pct) return;
      setTracePercent(pct, "number");
    });
    $("trace-move").addEventListener("click", () => {
      state.traceMove = !state.traceMove;
      paintChrome();
    });
    $("trace-visible").addEventListener("click", () => {
      state.trace.visible = !state.trace.visible;
      paintChrome();
      render();
    });
    $("fps").addEventListener("input", () => {
      state.fps = Math.max(1, Math.min(60, Number($("fps").value) || 1));
    });
    $("btn-play").addEventListener("click", () => {
      state.playing = !state.playing;
      paintChrome();
    });
    $("file-project").addEventListener("change", async () => {
      const file = $("file-project").files[0];
      if (file) await openText(await file.text(), null);
      $("file-project").value = "";
    });
    $("file-image").addEventListener("change", async () => {
      const file = $("file-image").files[0];
      if (file) await importImage(await readFile(file));
      $("file-image").value = "";
    });
    $("file-trace").addEventListener("change", async () => {
      const file = $("file-trace").files[0];
      if (file) await loadTrace(await readFile(file), true);
      $("file-trace").value = "";
    });
    window.addEventListener("dragover", (evt) => evt.preventDefault());
    window.addEventListener("drop", async (evt) => {
      evt.preventDefault();
      const file = evt.dataTransfer.files[0];
      if (!file) return;
      if (file.name.endsWith(".json")) await openText(await file.text(), null);
      else await loadTrace(await readFile(file), true);
    });
    requestAnimationFrame(function loop() {
      const now = performance.now();
      if (state.playing && frameCount() > 1 && now - lastPreview > 1000 / state.fps) {
        lastPreview = now;
        previewTick = (previewTick + 1) % frameCount();
        paintPreview();
      }
      requestAnimationFrame(loop);
    });
  }

  function moveFrame(dir) {
    const next = state.frame + dir;
    if (next < 0 || next >= frameCount()) return;
    pushUndo();
    syncTraceCount();
    state.layers.forEach((layer) => {
      const [frame] = layer.frames.splice(state.frame, 1);
      layer.frames.splice(next, 0, frame);
    });
    const [trace] = state.traces.splice(state.frame, 1);
    state.traces.splice(next, 0, trace);
    state.frame = next;
    state.trace = state.traces[state.frame];
    syncTraceControls();
    buildFrames();
    render();
  }

  function clampDim(value) {
    return Math.max(1, Math.min(512, Number(value) || 1));
  }

  function drawSmoke() {
    state.primary = [227, 123, 45, 255];
    const buf = currentBuffer();
    for (let y = 8; y < 24; y++) P.stamp(buf, state.width, state.height, 16, y, 1, state.primary);
    for (let x = 8; x < 24; x++) P.stamp(buf, state.width, state.height, x, 16, 1, [245, 240, 230, 255]);
    P.stamp(buf, state.width, state.height, 12, 12, 2, [196, 58, 42, 255]);
    P.stamp(buf, state.width, state.height, 20, 12, 2, [196, 58, 42, 255]);
    state.layers[0].frames.push(P.clone(buf));
    const dot = P.clone(buf);
    P.stamp(dot, state.width, state.height, 16, 20, 3, [240, 208, 96, 255]);
    state.layers[0].frames.push(dot);
    state.layers.push(makeLayer("Shade", state.layers[0].frames.map(() => blank())));
    P.stamp(state.layers[1].frames[0], state.width, state.height, 16, 22, 4, [88, 78, 70, 180]);
    buildLayers();
    buildFrames();
  }

  window.__smoke = async () => {
    fit();
    render();
    const rect = canvas.getBoundingClientRect();
    if (rect.width < 100 || rect.height < 100) throw new Error("canvas has no size");
    const fire = (type, x, y, button) => {
      canvas.dispatchEvent(new PointerEvent(type, {
        clientX: rect.left + state.panX + x * state.zoom + state.zoom / 2,
        clientY: rect.top + state.panY + y * state.zoom + state.zoom / 2,
        button: button || 0,
        buttons: type === "pointerup" ? 0 : 1,
        pointerId: 1,
        bubbles: true,
      }));
    };
    fit();
    const parked = state.panX;
    document.body.dispatchEvent(new KeyboardEvent("keydown", { key: " ", bubbles: true }));
    if (!hand) throw new Error("space did not arm canvas move");
    canvas.dispatchEvent(new PointerEvent("pointerdown", { clientX: rect.left + 80, clientY: rect.top + 80, button: 0, buttons: 1, pointerId: 3, bubbles: true }));
    canvas.dispatchEvent(new PointerEvent("pointermove", { clientX: rect.left + 200, clientY: rect.top + 80, button: 0, buttons: 1, pointerId: 3, bubbles: true }));
    canvas.dispatchEvent(new PointerEvent("pointerup", { clientX: rect.left + 200, clientY: rect.top + 80, button: 0, buttons: 0, pointerId: 3, bubbles: true }));
    document.body.dispatchEvent(new KeyboardEvent("keyup", { key: " ", bubbles: true }));
    if (hand) throw new Error("space stayed held");
    if (Math.abs(state.panX - parked) < 40) throw new Error("space drag left the canvas at " + state.panX);
    fit();
    setTool("pen");
    state.primary = [0, 200, 40, 255];
    fire("pointerdown", 2, 2);
    fire("pointermove", 7, 2);
    fire("pointerup", 7, 2);
    const drawn = P.get(currentBuffer(), 4, 2, state.width);
    if (drawn[1] < 100) throw new Error("pen stroke missed");
    undo();
    if (P.get(currentBuffer(), 4, 2, state.width)[3] !== 0) throw new Error("undo missed");
    redo();
    if (P.get(currentBuffer(), 4, 2, state.width)[1] < 100) throw new Error("redo missed");
    setTool("bucket");
    state.primary = [30, 30, 30, 255];
    fire("pointerdown", 0, 0);
    fire("pointerup", 0, 0);
    if (P.get(currentBuffer(), 0, 0, state.width)[0] !== 30) throw new Error("bucket missed");
    $("frame-dup").click();
    if (frameCount() !== 2) throw new Error("frame duplicate missed");
    const film = document.querySelector(".film").getBoundingClientRect();
    for (const id of ["frame-add", "frame-dup", "frame-left", "frame-right", "frame-del"]) {
      const box = $(id).getBoundingClientRect();
      if (box.height < 8 || box.top < film.top - 1 || box.bottom > film.bottom + 1) {
        throw new Error(id + " is clipped");
      }
    }
    setTool("select");
    fire("pointerdown", 1, 1);
    fire("pointermove", 5, 4);
    fire("pointerup", 5, 4);
    if (!state.selection || state.selection.w < 2 || state.selection.h < 2) throw new Error("selection missed");
    const saved = serialize();
    resetDocument(32, 32, "sprite");
    await openText(saved);
    if (P.get(currentBuffer(), 4, 2, state.width)[1] < 100) throw new Error("save roundtrip missed");
    if (frameCount() !== 2) throw new Error("save dropped a frame");
    resetDocument(32, 32, "mascot");
    drawSmoke();
    const c = document.createElement("canvas");
    c.width = 64;
    c.height = 64;
    const g = c.getContext("2d");
    g.strokeStyle = "rgba(80,160,255,0.95)";
    g.lineWidth = 3;
    g.strokeRect(6, 6, 52, 52);
    g.beginPath();
    g.moveTo(10, 54);
    g.lineTo(54, 10);
    g.stroke();
    await loadTrace(c.toDataURL("image/png"), true);
    state.trace.opacity = 0.55;
    if (!state.trace.image) throw new Error("trace image did not load");
    const wide = document.createElement("canvas");
    wide.width = 2000;
    wide.height = 400;
    wide.getContext("2d").fillRect(0, 0, 20, 20);
    await loadTrace(wide.toDataURL("image/png"), true);
    const fitted = state.trace.scale;
    state.traceMove = false;
    paintChrome();
    if (!$("trace-scale").disabled) throw new Error("scale unlocked while move trace is off");
    $("trace-scale").value = "0";
    $("trace-scale").dispatchEvent(new Event("input", { bubbles: true }));
    if (state.trace.scale !== fitted) throw new Error("locked scale changed");
    state.traceMove = true;
    paintChrome();
    $("trace-scale").dispatchEvent(new Event("input", { bubbles: true }));
    if (!(state.trace.scale < fitted / 100)) throw new Error("trace scale min is still " + state.trace.scale + ", fit was " + fitted);
    state.traceMove = false;
    paintChrome();
    const swatch = document.createElement("canvas");
    swatch.width = 32;
    swatch.height = 32;
    const sg = swatch.getContext("2d");
    sg.fillStyle = "rgb(10, 180, 40)";
    sg.fillRect(0, 0, 32, 32);
    await loadTrace(swatch.toDataURL("image/png"), true);
    state.trace.opacity = 0.2;
    setTool("picker");
    state.primary = [0, 0, 0, 255];
    fire("pointerdown", 4, 4);
    if (state.primary[0] > 40 || state.primary[1] < 150 || state.primary[2] > 80) {
      throw new Error("trace sample missed " + state.primary.join(","));
    }
    const keptFrame = state.frame;
    const keptTrace = state.trace.dataUrl;
    $("frame-add").click();
    if (state.trace.dataUrl) throw new Error("added frame reused the previous trace");
    if (state.traces[keptFrame].dataUrl !== keptTrace) throw new Error("the other frame lost its trace");
    const other = document.createElement("canvas");
    other.width = 8;
    other.height = 8;
    const og = other.getContext("2d");
    og.fillStyle = "rgb(20, 40, 200)";
    og.fillRect(0, 0, 8, 8);
    await loadTrace(other.toDataURL("image/png"), true);
    if (!state.trace.dataUrl || state.trace.dataUrl === state.traces[keptFrame].dataUrl) throw new Error("frames still share one trace");
    state.frame = 0;
    syncTraceCount();
    traceSample = null;
    await loadTrace(c.toDataURL("image/png"), true);
    state.trace.opacity = 0.55;
    const exported = P.composite(state.layers, 0, state.width, state.height, false, 0);
    if (exported[3] !== 0) throw new Error("trace leaked into the export");
    const gifFrame = new Uint8ClampedArray(4 * 4 * 4);
    P.set(gifFrame, 0, 0, 4, [200, 10, 10, 255]);
    P.set(gifFrame, 1, 0, 4, [10, 180, 20, 255]);
    const gif = window.Gif.encode([gifFrame], 4, 4, 8);
    const decoded = await loadImage("data:image/gif;base64," + bytesToB64(gif));
    const probe = document.createElement("canvas");
    probe.width = 4;
    probe.height = 4;
    probe.getContext("2d").drawImage(decoded, 0, 0);
    const got = probe.getContext("2d").getImageData(0, 0, 1, 1).data;
    if (got[0] < 150 || got[1] > 40) throw new Error("gif decode failed " + got.join(","));
    setTool("pen");
    fit();
    render();
    await new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve)));
  };

  window.__editor = {
    state,
    isDirty() {
      return state.dirty;
    },
    saveProject,
    setTool,
    stamp(x, y, color, size) {
      P.stamp(currentBuffer(), state.width, state.height, x, y, size || 1, color);
      render();
    },
  };

  resetDocument(32, 32, "sprite");
  buildPalette();
  bind();
  paintChrome();
  requestAnimationFrame(() => {
    fit();
    render();
  });
})();
