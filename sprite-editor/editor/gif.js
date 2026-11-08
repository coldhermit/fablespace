// A small GIF89a writer. One global palette, looping animation, one transparent index.
(function (root) {
  function quantize(frames) {
    const colors = new Map();
    colors.set("0,0,0,0", 0);
    const palette = [[0, 0, 0]];
    function nearest(r, g, b) {
      let best = 1;
      let bestD = Infinity;
      for (let i = 1; i < palette.length; i++) {
        const c = palette[i];
        const d = (c[0] - r) * (c[0] - r) + (c[1] - g) * (c[1] - g) + (c[2] - b) * (c[2] - b);
        if (d < bestD) {
          bestD = d;
          best = i;
        }
      }
      return best;
    }
    for (const frame of frames) {
      for (let i = 0; i < frame.length; i += 4) {
        if (frame[i + 3] < 128) continue;
        const key = frame[i] + "," + frame[i + 1] + "," + frame[i + 2];
        if (colors.has(key)) continue;
        if (palette.length < 256) {
          colors.set(key, palette.length);
          palette.push([frame[i], frame[i + 1], frame[i + 2]]);
        }
      }
    }
    const indexed = frames.map((frame) => {
      const out = new Uint8Array(frame.length / 4);
      for (let p = 0, i = 0; i < frame.length; i += 4, p++) {
        if (frame[i + 3] < 128) {
          out[p] = 0;
          continue;
        }
        const key = frame[i] + "," + frame[i + 1] + "," + frame[i + 2];
        out[p] = colors.has(key) ? colors.get(key) : nearest(frame[i], frame[i + 1], frame[i + 2]);
      }
      return out;
    });
    let size = 2;
    while (size < palette.length) size *= 2;
    while (palette.length < size) palette.push([0, 0, 0]);
    return { palette, indexed, size };
  }

  function lzw(indices, minCodeSize) {
    const clear = 1 << minCodeSize;
    const eoi = clear + 1;
    let codeSize = minCodeSize + 1;
    let next = eoi + 1;
    let dict = new Map();
    function reset() {
      dict = new Map();
      for (let i = 0; i < clear; i++) dict.set(String.fromCharCode(i), i);
      codeSize = minCodeSize + 1;
      next = eoi + 1;
    }
    reset();
    const bytes = [];
    let acc = 0;
    let bits = 0;
    function write(code) {
      acc |= code << bits;
      bits += codeSize;
      while (bits >= 8) {
        bytes.push(acc & 255);
        acc >>= 8;
        bits -= 8;
      }
    }
    write(clear);
    if (!indices.length) {
      write(eoi);
      if (bits) bytes.push(acc & 255);
      return bytes;
    }
    let prefix = String.fromCharCode(indices[0]);
    for (let i = 1; i < indices.length; i++) {
      const k = String.fromCharCode(indices[i]);
      const combined = prefix + k;
      if (dict.has(combined)) {
        prefix = combined;
        continue;
      }
      write(dict.get(prefix));
      if (next < 4096) {
        dict.set(combined, next++);
        if (next === 1 << codeSize && codeSize < 12) codeSize++;
      } else {
        write(clear);
        reset();
      }
      prefix = k;
    }
    write(dict.get(prefix));
    write(eoi);
    if (bits) bytes.push(acc & 255);
    return bytes;
  }

  function subBlocks(bytes) {
    const out = [];
    for (let i = 0; i < bytes.length; i += 255) {
      const n = Math.min(255, bytes.length - i);
      out.push(n);
      for (let j = 0; j < n; j++) out.push(bytes[i + j]);
    }
    out.push(0);
    return out;
  }

  function encode(frames, width, height, fps) {
    const { palette, indexed, size } = quantize(frames);
    const minCodeSize = Math.max(2, Math.round(Math.log2(size)));
    const delay = Math.max(2, Math.round(100 / Math.max(1, fps)));
    const colorBits = Math.round(Math.log2(size)) - 1;
    const bytes = [];
    const push = (...ns) => ns.forEach((n) => bytes.push(n & 255));
    const str = (s) => {
      for (let i = 0; i < s.length; i++) bytes.push(s.charCodeAt(i));
    };
    str("GIF89a");
    push(width, width >> 8, height, height >> 8, 0x80 | (colorBits << 4) | colorBits, 0, 0);
    for (const c of palette) push(c[0], c[1], c[2]);
    push(0x21, 0xff, 11);
    str("NETSCAPE2.0");
    push(3, 1, 0, 0, 0);
    for (const frame of indexed) {
      push(0x21, 0xf9, 4, 0x09, delay, delay >> 8, 0, 0);
      push(0x2c, 0, 0, 0, 0, width, width >> 8, height, height >> 8, 0);
      push(minCodeSize);
      subBlocks(lzw(frame, minCodeSize)).forEach((n) => bytes.push(n));
    }
    push(0x3b);
    return new Uint8Array(bytes);
  }

  root.Gif = { encode };
  if (typeof module !== "undefined" && module.exports) module.exports = { encode };
})(typeof window !== "undefined" ? window : globalThis);
