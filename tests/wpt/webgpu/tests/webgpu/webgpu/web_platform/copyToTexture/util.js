/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ import { TexelView } from '../../util/texture/texel_view.js';

// None of the dst texture format is 'uint' or 'sint', so we can always use float value.
const kColors = {
  Red: { R: 1.0, G: 0.0, B: 0.0, A: 1.0 },
  Green: { R: 0.0, G: 1.0, B: 0.0, A: 1.0 },
  Blue: { R: 0.0, G: 0.0, B: 1.0, A: 1.0 },
  Black: { R: 0.0, G: 0.0, B: 0.0, A: 1.0 },
  White: { R: 1.0, G: 1.0, B: 1.0, A: 1.0 },
  SemitransparentWhite: { R: 1.0, G: 1.0, B: 1.0, A: 0.6 },
};

export const kTestColorsOpaque = [
  kColors.Red,
  kColors.Green,
  kColors.Blue,
  kColors.Black,
  kColors.White,
];

export const kTestColorsAll = [...kTestColorsOpaque, kColors.SemitransparentWhite];

export function makeTestColorsTexelView({
  testColors,
  format,
  width,
  height,
  premultiplied,
  flipY,
}) {
  return TexelView.fromTexelsAsColors(format, coords => {
    const y = flipY ? height - coords.y - 1 : coords.y;
    const pixelPos = y * width + coords.x;
    const currentPixel = testColors[pixelPos % testColors.length];

    if (premultiplied && currentPixel.A !== 1.0) {
      return {
        R: currentPixel.R * currentPixel.A,
        G: currentPixel.G * currentPixel.A,
        B: currentPixel.B * currentPixel.A,
        A: currentPixel.A,
      };
    } else {
      return currentPixel;
    }
  });
}
