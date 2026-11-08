const assert = require("assert");
const P = require("../editor/pixels");
const Gif = require("../editor/gif");
const TraceScale = require("../editor/trace-scale");

function colorAt(data, x, y, w) {
  return P.get(data, x, y, w).join(",");
}

{
  const pts = P.line(0, 0, 4, 0);
  assert.deepStrictEqual(pts, [[0, 0], [1, 0], [2, 0], [3, 0], [4, 0]]);
  const diag = P.line(0, 0, 2, 2);
  assert.deepStrictEqual(diag, [[0, 0], [1, 1], [2, 2]]);
}

{
  assert.strictEqual(P.stampOrigin(1), 0);
  assert.strictEqual(P.stampOrigin(2), -1);
  assert.strictEqual(P.stampOrigin(3), -1);
  assert.strictEqual(P.stampOrigin(4), -2);
  const data = new Uint8ClampedArray(4 * 4 * 4);
  P.stamp(data, 4, 4, 2, 2, 2, [9, 0, 0, 255]);
  assert.strictEqual(colorAt(data, 1, 1, 4), "9,0,0,255");
  assert.strictEqual(colorAt(data, 2, 2, 4), "9,0,0,255");
  assert.strictEqual(colorAt(data, 0, 2, 4), "0,0,0,0");
  assert.strictEqual(colorAt(data, 3, 2, 4), "0,0,0,0");
}

{
  const data = new Uint8ClampedArray(5 * 5 * 4);
  P.stamp(data, 5, 5, 0, 2, 1, [1, 0, 0, 255]);
  P.stamp(data, 5, 5, 1, 2, 1, [1, 0, 0, 255]);
  P.flood(data, 5, 5, 3, 2, [0, 2, 0, 255]);
  assert.strictEqual(colorAt(data, 3, 2, 5), "0,2,0,255");
  assert.strictEqual(colorAt(data, 0, 2, 5), "1,0,0,255");
  assert.strictEqual(colorAt(data, 4, 4, 5), "0,2,0,255");
}

{
  const data = new Uint8ClampedArray(3 * 1 * 4);
  P.set(data, 0, 0, 3, [4, 0, 0, 255]);
  const moved = P.shift(data, 3, 1, 1, 0, true);
  assert.strictEqual(colorAt(moved, 1, 0, 3), "4,0,0,255");
  assert.strictEqual(colorAt(moved, 0, 0, 3), "0,0,0,0");
  const wrapped = P.shift(data, 3, 1, -1, 0, true);
  assert.strictEqual(colorAt(wrapped, 2, 0, 3), "4,0,0,255");
}

{
  const pts = P.rectPoints(1, 1, 4, 3, 2);
  assert.ok(pts.some((p) => p[0] === 1 && p[1] === 1));
  assert.ok(pts.some((p) => p[0] === 4 && p[1] === 3));
  assert.ok(pts.some((p) => p[0] === 2 && p[1] === 2));
  assert.ok(!pts.some((p) => p[0] === 0 || p[1] === 0));
}

{
  const [x1, y1] = P.snapLine(0, 0, 8, 1);
  assert.strictEqual(x1, 8);
  assert.strictEqual(y1, 0);
  const square = P.box(0, 0, 4, 1, true);
  assert.deepStrictEqual(square, [0, 0, 4, 4]);
}

{
  const w = 2;
  const h = 2;
  const lower = new Uint8ClampedArray([255, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
  const upper = new Uint8ClampedArray([0, 0, 255, 128, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
  const out = P.composite([
    { visible: true, opacity: 1, frames: [lower] },
    { visible: true, opacity: 1, frames: [upper] },
  ], 0, w, h, false, 1);
  assert.ok(out[2] > 80 && out[0] > 80);
  const dim = P.composite([
    { visible: true, opacity: 1, frames: [lower] },
    { visible: true, opacity: 1, frames: [upper] },
  ], 0, w, h, true, 0);
  assert.ok(dim[0] > dim[2]);
}

function decodeGif(bytes) {
  assert.strictEqual(String.fromCharCode(...bytes.slice(0, 6)), "GIF89a");
  assert.strictEqual(bytes[bytes.length - 1], 0x3b);
  let p = 6;
  const width = bytes[p] | (bytes[p + 1] << 8);
  const height = bytes[p + 2] | (bytes[p + 3] << 8);
  const packed = bytes[p + 4];
  p += 7;
  const tableSize = 1 << ((packed & 7) + 1);
  const palette = [];
  for (let i = 0; i < tableSize; i++) {
    palette.push([bytes[p], bytes[p + 1], bytes[p + 2]]);
    p += 3;
  }
  const frames = [];
  while (p < bytes.length && bytes[p] !== 0x3b) {
    if (bytes[p] === 0x21) {
      p += 2;
      const block = bytes[p++];
      p += block;
      while (bytes[p] !== 0) p += bytes[p] + 1;
      p += 1;
      continue;
    }
    if (bytes[p] !== 0x2c) throw new Error("expected image at " + p);
    p += 9;
    const packedImage = bytes[p++];
    if (packedImage & 0x80) throw new Error("local palette");
    const minCodeSize = bytes[p++];
    const lzw = [];
    while (bytes[p] !== 0) {
      const n = bytes[p++];
      for (let i = 0; i < n; i++) lzw.push(bytes[p++]);
    }
    p += 1;
    frames.push(lzwDecode(lzw, minCodeSize, width * height).map((index) => {
      if (index === 0) return [0, 0, 0, 0];
      const color = palette[index];
      return [color[0], color[1], color[2], 255];
    }));
  }
  return { width, height, frames };
}

function lzwDecode(data, minCodeSize, count) {
  const clear = 1 << minCodeSize;
  const eoi = clear + 1;
  let codeSize = minCodeSize + 1;
  let next = eoi + 1;
  let table = [];
  const reset = () => {
    table = [];
    for (let i = 0; i < clear; i++) table[i] = [i];
    codeSize = minCodeSize + 1;
    next = eoi + 1;
  };
  reset();
  let acc = 0;
  let bits = 0;
  let pos = 0;
  const read = () => {
    while (bits < codeSize) {
      if (pos >= data.length) return eoi;
      acc |= data[pos++] << bits;
      bits += 8;
    }
    const mask = (1 << codeSize) - 1;
    const code = acc & mask;
    acc >>= codeSize;
    bits -= codeSize;
    return code;
  };
  let code = read();
  if (code === clear) code = read();
  const out = [];
  if (code === eoi) return out;
  let prev = table[code].slice();
  out.push(...prev);
  while (out.length < count) {
    code = read();
    if (code === clear) {
      reset();
      code = read();
      if (code === eoi) break;
      prev = table[code].slice();
      out.push(...prev);
      continue;
    }
    if (code === eoi) break;
    let entry;
    if (code < next && table[code]) entry = table[code].slice();
    else entry = prev.concat(prev[0]);
    out.push(...entry);
    if (next < 4096) {
      table[next++] = prev.concat(entry[0]);
      if (next === 1 << codeSize && codeSize < 12) codeSize++;
    }
    prev = entry;
  }
  return out.slice(0, count);
}

{
  const frame = new Uint8ClampedArray(2 * 2 * 4);
  frame.set([200, 10, 10, 255, 200, 10, 10, 255, 0, 0, 0, 0, 10, 20, 200, 255]);
  const other = new Uint8ClampedArray(frame);
  other.set([10, 180, 20, 255, 10, 180, 20, 255, 10, 180, 20, 255, 10, 180, 20, 255]);
  const gif = Gif.encode([frame, other], 2, 2, 10);
  const decoded = decodeGif(gif);
  assert.strictEqual(decoded.width, 2);
  assert.strictEqual(decoded.height, 2);
  assert.strictEqual(decoded.frames.length, 2);
  assert.deepStrictEqual(decoded.frames[0][0], [200, 10, 10, 255]);
  assert.deepStrictEqual(decoded.frames[0][2], [0, 0, 0, 0]);
  assert.deepStrictEqual(decoded.frames[0][3], [10, 20, 200, 255]);
  assert.deepStrictEqual(decoded.frames[1][0], [10, 180, 20, 255]);
}

{
  const fit = TraceScale.fitScale(32, 32, 2000, 1200);
  assert.ok(Math.abs(fit - 32 / 2000) < 1e-9);
  const smallest = fit * (TraceScale.percentFromSlider(0) / 100);
  assert.ok(smallest < fit / 100, "smallest scale should be under 1% of the fitted size");
  const back = TraceScale.percentFromSlider(TraceScale.sliderFromPercent(100));
  assert.ok(Math.abs(back - 100) < 1.5);
  assert.ok(TraceScale.percentFromSlider(0) <= TraceScale.MIN);
}

console.log("pixel and gif tests passed");
