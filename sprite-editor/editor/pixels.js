// Pixel buffers are the picture. A canvas is only a place to show them.
(function (root) {
  function clone(data) {
    return new Uint8ClampedArray(data);
  }

  function index(x, y, w) {
    return (y * w + x) * 4;
  }

  function get(data, x, y, w) {
    const i = index(x, y, w);
    return [data[i], data[i + 1], data[i + 2], data[i + 3]];
  }

  function set(data, x, y, w, color) {
    const i = index(x, y, w);
    data[i] = color[0];
    data[i + 1] = color[1];
    data[i + 2] = color[2];
    data[i + 3] = color[3];
  }

  function same(a, b) {
    return a[0] === b[0] && a[1] === b[1] && a[2] === b[2] && a[3] === b[3];
  }

  function stampOrigin(size) {
    return 0 - (size >> 1);
  }

  function stamp(data, w, h, x, y, size, color) {
    const o = stampOrigin(size);
    for (let dy = 0; dy < size; dy++) {
      for (let dx = 0; dx < size; dx++) {
        const px = x + o + dx;
        const py = y + o + dy;
        if (px >= 0 && py >= 0 && px < w && py < h) set(data, px, py, w, color);
      }
    }
  }

  function line(x0, y0, x1, y1) {
    const pts = [];
    let dx = Math.abs(x1 - x0);
    let dy = Math.abs(y1 - y0);
    const sx = x0 < x1 ? 1 : -1;
    const sy = y0 < y1 ? 1 : -1;
    let err = dx - dy;
    for (;;) {
      pts.push([x0, y0]);
      if (x0 === x1 && y0 === y1) break;
      const e2 = 2 * err;
      if (e2 > -dy) {
        err -= dy;
        x0 += sx;
      }
      if (e2 < dx) {
        err += dx;
        y0 += sy;
      }
    }
    return pts;
  }

  function snapLine(x0, y0, x1, y1) {
    const dx = x1 - x0;
    const dy = y1 - y0;
    if (dx === 0 && dy === 0) return [x1, y1];
    const len = Math.hypot(dx, dy);
    const slopes = [
      [1, 0],
      [2, 1],
      [3, 1],
      [1, 1],
      [1, 2],
      [1, 3],
      [0, 1],
    ];
    let best = null;
    let bestDot = -Infinity;
    for (const [a, b] of slopes) {
      const signs = [];
      if (a === 0) signs.push([1, 1], [1, -1]);
      else if (b === 0) signs.push([1, 1], [-1, 1]);
      else signs.push([1, 1], [1, -1], [-1, 1], [-1, -1]);
      for (const [sx, sy] of signs) {
        const vx = a * sx;
        const vy = b * sy;
        const vl = Math.hypot(vx, vy);
        const dot = (dx * vx + dy * vy) / (len * vl);
        if (dot > bestDot) {
          bestDot = dot;
          best = [vx / vl, vy / vl];
        }
      }
    }
    const dist = Math.round(len);
    return [x0 + Math.round(best[0] * dist), y0 + Math.round(best[1] * dist)];
  }

  function box(x0, y0, x1, y1, square) {
    if (!square) return [x0, y0, x1, y1];
    const dx = x1 - x0;
    const dy = y1 - y0;
    const s = Math.max(Math.abs(dx), Math.abs(dy));
    const sx = dx === 0 ? 1 : Math.sign(dx);
    const sy = dy === 0 ? 1 : Math.sign(dy);
    return [x0, y0, x0 + sx * s, y0 + sy * s];
  }

  function rectPoints(x0, y0, x1, y1, thickness) {
    let left = Math.min(x0, x1);
    let right = Math.max(x0, x1);
    let top = Math.min(y0, y1);
    let bottom = Math.max(y0, y1);
    const pts = [];
    const seen = new Set();
    const add = (x, y) => {
      const k = x + "," + y;
      if (seen.has(k)) return;
      seen.add(k);
      pts.push([x, y]);
    };
    for (let t = 0; t < thickness; t++) {
      if (left > right || top > bottom) break;
      for (let x = left; x <= right; x++) {
        add(x, top);
        add(x, bottom);
      }
      for (let y = top + 1; y <= bottom - 1; y++) {
        add(left, y);
        add(right, y);
      }
      left++;
      right--;
      top++;
      bottom--;
    }
    return pts;
  }

  function ellipsePoints(x0, y0, x1, y1, thickness) {
    const pts = [];
    const seen = new Set();
    const add = (x, y) => {
      const k = x + "," + y;
      if (seen.has(k)) return;
      seen.add(k);
      pts.push([x, y]);
    };
    for (let t = 0; t < thickness; t++) {
      const left = Math.min(x0, x1) + t;
      const right = Math.max(x0, x1) - t;
      const top = Math.min(y0, y1) + t;
      const bottom = Math.max(y0, y1) - t;
      if (left > right || top > bottom) break;
      const rx = (right - left) / 2;
      const ry = (bottom - top) / 2;
      const cx = (left + right) / 2;
      const cy = (top + bottom) / 2;
      if (rx < 0.5 || ry < 0.5) {
        add(Math.round(cx), Math.round(cy));
        continue;
      }
      const steps = Math.max(12, Math.ceil(2 * Math.PI * Math.max(rx, ry) * 2));
      let prev = null;
      for (let i = 0; i <= steps; i++) {
        const a = (i / steps) * Math.PI * 2;
        const x = Math.round(cx + Math.cos(a) * rx);
        const y = Math.round(cy + Math.sin(a) * ry);
        if (prev) {
          for (const p of line(prev[0], prev[1], x, y)) add(p[0], p[1]);
        }
        prev = [x, y];
      }
    }
    return pts;
  }

  function plot(data, w, h, points, color) {
    for (const [x, y] of points) {
      if (x >= 0 && y >= 0 && x < w && y < h) set(data, x, y, w, color);
    }
  }

  function flood(data, w, h, x, y, color) {
    if (x < 0 || y < 0 || x >= w || y >= h) return;
    const target = get(data, x, y, w);
    if (same(target, color)) return;
    const stack = [[x, y]];
    const seen = new Uint8Array(w * h);
    while (stack.length) {
      const [cx, cy] = stack.pop();
      if (cx < 0 || cy < 0 || cx >= w || cy >= h) continue;
      const id = cy * w + cx;
      if (seen[id]) continue;
      if (!same(get(data, cx, cy, w), target)) continue;
      seen[id] = 1;
      set(data, cx, cy, w, color);
      stack.push([cx + 1, cy], [cx - 1, cy], [cx, cy + 1], [cx, cy - 1]);
    }
  }

  function replaceColor(data, from, to) {
    for (let i = 0; i < data.length; i += 4) {
      if (data[i] === from[0] && data[i + 1] === from[1] && data[i + 2] === from[2] && data[i + 3] === from[3]) {
        data[i] = to[0];
        data[i + 1] = to[1];
        data[i + 2] = to[2];
        data[i + 3] = to[3];
      }
    }
  }

  function shift(src, w, h, dx, dy, wrap) {
    const out = new Uint8ClampedArray(src.length);
    for (let y = 0; y < h; y++) {
      for (let x = 0; x < w; x++) {
        let sx = x - dx;
        let sy = y - dy;
        if (wrap) {
          sx = ((sx % w) + w) % w;
          sy = ((sy % h) + h) % h;
        } else if (sx < 0 || sy < 0 || sx >= w || sy >= h) {
          continue;
        }
        const si = index(sx, sy, w);
        const di = index(x, y, w);
        out[di] = src[si];
        out[di + 1] = src[si + 1];
        out[di + 2] = src[si + 2];
        out[di + 3] = src[si + 3];
      }
    }
    return out;
  }

  function rgbToHsl(r, g, b) {
    r /= 255;
    g /= 255;
    b /= 255;
    const max = Math.max(r, g, b);
    const min = Math.min(r, g, b);
    let h = 0;
    const l = (max + min) / 2;
    const d = max - min;
    if (d === 0) return [0, 0, l];
    const s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
    if (max === r) h = (g - b) / d + (g < b ? 6 : 0);
    else if (max === g) h = (b - r) / d + 2;
    else h = (r - g) / d + 4;
    return [h / 6, s, l];
  }

  function hue(p, q, t) {
    if (t < 0) t += 1;
    if (t > 1) t -= 1;
    if (t < 1 / 6) return p + (q - p) * 6 * t;
    if (t < 1 / 2) return q;
    if (t < 2 / 3) return p + (q - p) * (2 / 3 - t) * 6;
    return p;
  }

  function hslToRgb(h, s, l) {
    if (s === 0) {
      const v = Math.round(l * 255);
      return [v, v, v];
    }
    const q = l < 0.5 ? l * (1 + s) : l + s - l * s;
    const p = 2 * l - q;
    return [
      Math.round(hue(p, q, h + 1 / 3) * 255),
      Math.round(hue(p, q, h) * 255),
      Math.round(hue(p, q, h - 1 / 3) * 255),
    ];
  }

  function lightenAt(data, w, h, x, y, amount) {
    if (x < 0 || y < 0 || x >= w || y >= h) return;
    const c = get(data, x, y, w);
    if (c[3] === 0) return;
    const hsl = rgbToHsl(c[0], c[1], c[2]);
    hsl[2] = Math.min(1, Math.max(0, hsl[2] + amount));
    const rgb = hslToRgb(hsl[0], hsl[1], hsl[2]);
    set(data, x, y, w, [rgb[0], rgb[1], rgb[2], c[3]]);
  }

  function flip(src, w, h, horizontal) {
    const out = new Uint8ClampedArray(src.length);
    for (let y = 0; y < h; y++) {
      for (let x = 0; x < w; x++) {
        const sx = horizontal ? w - 1 - x : x;
        const sy = horizontal ? y : h - 1 - y;
        const si = index(sx, sy, w);
        const di = index(x, y, w);
        out[di] = src[si];
        out[di + 1] = src[si + 1];
        out[di + 2] = src[si + 2];
        out[di + 3] = src[si + 3];
      }
    }
    return out;
  }

  function rotate(src, w, h) {
    const out = new Uint8ClampedArray(h * w * 4);
    for (let y = 0; y < h; y++) {
      for (let x = 0; x < w; x++) {
        const si = index(x, y, w);
        const nx = h - 1 - y;
        const ny = x;
        const di = index(nx, ny, h);
        out[di] = src[si];
        out[di + 1] = src[si + 1];
        out[di + 2] = src[si + 2];
        out[di + 3] = src[si + 3];
      }
    }
    return out;
  }

  function blit(dest, dw, dh, src, sw, sh, ox, oy) {
    for (let y = 0; y < sh; y++) {
      for (let x = 0; x < sw; x++) {
        const dx = ox + x;
        const dy = oy + y;
        if (dx < 0 || dy < 0 || dx >= dw || dy >= dh) continue;
        const c = get(src, x, y, sw);
        if (c[3] === 0) continue;
        set(dest, dx, dy, dw, c);
      }
    }
  }

  function composite(layers, frame, w, h, dimOthers, current) {
    const out = new Uint8ClampedArray(w * h * 4);
    layers.forEach((layer, li) => {
      if (!layer.visible) return;
      const src = layer.frames[frame];
      let alpha = layer.opacity;
      if (dimOthers && li !== current) alpha *= 0.2;
      for (let i = 0; i < src.length; i += 4) {
        const sa = (src[i + 3] / 255) * alpha;
        if (sa <= 0) continue;
        const da = out[i + 3] / 255;
        const oa = sa + da * (1 - sa);
        if (oa <= 0) continue;
        out[i] = Math.round((src[i] * sa + out[i] * da * (1 - sa)) / oa);
        out[i + 1] = Math.round((src[i + 1] * sa + out[i + 1] * da * (1 - sa)) / oa);
        out[i + 2] = Math.round((src[i + 2] * sa + out[i + 2] * da * (1 - sa)) / oa);
        out[i + 3] = Math.round(oa * 255);
      }
    });
    return out;
  }

  const api = {
    clone,
    index,
    get,
    set,
    same,
    stampOrigin,
    stamp,
    line,
    snapLine,
    box,
    rectPoints,
    ellipsePoints,
    plot,
    flood,
    replaceColor,
    shift,
    lightenAt,
    flip,
    rotate,
    blit,
    composite,
  };
  root.Pixels = api;
  if (typeof module !== "undefined" && module.exports) module.exports = api;
})(typeof window !== "undefined" ? window : globalThis);
