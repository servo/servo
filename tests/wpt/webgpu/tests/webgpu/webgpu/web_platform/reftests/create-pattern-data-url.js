/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/ // creates a 4x4 pattern
export default function createPatternDataURL() {const patternSize = 4;const ctx = document.createElement('canvas').getContext('2d');
  ctx.canvas.width = patternSize;
  ctx.canvas.height = patternSize;

  const b = [0, 0, 0, 255];
  const t = [0, 0, 0, 0];
  const r = [255, 0, 0, 255];
  const g = [0, 255, 0, 255];

  const imageData = new ImageData(patternSize, patternSize);

  imageData.data.set([
  b, t, t, r,
  t, b, g, t,
  t, r, b, t,
  g, t, t, b].
  flat());
  ctx.putImageData(imageData, 0, 0);
  return { patternSize, imageData, dataURL: ctx.canvas.toDataURL() };
}