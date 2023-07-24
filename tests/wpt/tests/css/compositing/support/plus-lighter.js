import { clamp01, multiplyAlpha, unmultiplyAlpha } from "./utils.js";

export function plusLighter(pixels) {
  if (pixels.length === 1) return pixels[0];

  return pixels.reduce((destination, source) => {
    const premultipliedSource = multiplyAlpha(source);
    const premultipliedDestination = multiplyAlpha(destination);
    const premultipliedResult = premultipliedDestination.map((channel, i) =>
      clamp01(channel + premultipliedSource[i])
    );
    return unmultiplyAlpha(premultipliedResult);
  });
}

export const tests = [
  // Each test is a list of colors to composite.
  // Each color is [r, g, b, a], unmultiplied, in the range 0-1.
  [
    [1, 0, 0, 0.5],
    [0, 0, 1, 0.5],
  ],
  [
    [1, 0, 0, 0.25],
    [0, 0, 1, 0.25],
  ],
  [
    [0.5, 0, 0, 0.5],
    [0, 0, 1, 0.5],
  ],
  // Test clamping
  [
    [1, 0, 0, 1],
    [0, 0, 1, 1],
  ],
  // Test more than two elements
  [
    [1, 0, 0, 0.25],
    [0, 0, 1, 0.25],
    [0, 1, 0, 0.25],
    [0.5, 0.4, 0.25, 0.25],
  ],
  // Test a single element
  [
    [0.5, 0, 0, 0.25],
  ],
];
