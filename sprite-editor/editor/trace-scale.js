// Trace scale is a percent of "fit the whole picture inside the sprite".
// The slider is logarithmic so the small end is actually reachable.
(function (root) {
  const MIN = 0.05;
  const MAX = 800;

  function fitScale(spriteW, spriteH, imageW, imageH) {
    if (!imageW || !imageH) return 1;
    return Math.min(spriteW / imageW, spriteH / imageH);
  }

  function percentFromSlider(value) {
    const t = Math.min(1, Math.max(0, Number(value) / 1000));
    return MIN * Math.pow(MAX / MIN, t);
  }

  function sliderFromPercent(pct) {
    const clamped = Math.min(MAX, Math.max(MIN, pct));
    const t = Math.log(clamped / MIN) / Math.log(MAX / MIN);
    return Math.round(t * 1000);
  }

  const api = { MIN, MAX, fitScale, percentFromSlider, sliderFromPercent };
  root.TraceScale = api;
  if (typeof module !== "undefined" && module.exports) module.exports = api;
})(typeof window !== "undefined" ? window : globalThis);
