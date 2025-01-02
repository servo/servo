/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = 'Color space conversion helpers';import { Fixture } from '../../../common/framework/fixture.js';
import { makeTestGroup } from '../../../common/framework/test_group.js';
import { ErrorWithExtra } from '../../../common/util/util.js';
import { makeInPlaceColorConversion } from '../color_space_conversion.js';
import { clamp } from '../math.js';

import { TexelView } from './texel_view.js';
import { findFailedPixels } from './texture_ok.js';

const kTestColors = [
[0xff, 0, 0],
[0, 0xff, 0],
[0, 0, 0xff],
[0x80, 0x80, 0],
[0, 0x80, 0x80],
[0x80, 0, 0x80]];


function floatToU8(v) {
  return clamp(Math.round(v * 255), { min: 0, max: 255 });
}

export const g = makeTestGroup(Fixture);

g.test('util_matches_2d_canvas').
desc(`Test color space conversion helpers matches canvas 2d's color space conversion`).
params((u) =>
u.combineWithParams([
{ srcColorSpace: 'srgb', dstColorSpace: 'display-p3' },
{ srcColorSpace: 'display-p3', dstColorSpace: 'srgb' }]
)
).
fn((t) => {
  const { srcColorSpace, dstColorSpace } = t.params;

  // putImageData an ImageData(srcColorSpace) in to a canvas2D(dstColorSpace)
  // then call getImageData. This will convert the colors via the canvas 2D API
  const width = kTestColors.length;
  const height = 1;
  const imgData = new ImageData(
    new Uint8ClampedArray(kTestColors.map((v) => [...v, 255]).flat()),
    width,
    height,
    { colorSpace: srcColorSpace }
  );
  const ctx = new OffscreenCanvas(width, height).getContext('2d', {
    colorSpace: dstColorSpace
  });
  ctx.putImageData(imgData, 0, 0);
  const expectedData = ctx.getImageData(0, 0, width, height).data;

  const conversionFn = makeInPlaceColorConversion({
    srcPremultiplied: false,
    dstPremultiplied: false,
    srcColorSpace,
    dstColorSpace
  });

  // Convert the data via our conversion functions
  const convertedData = new Uint8ClampedArray(
    kTestColors.
    map((color) => {
      const [R, G, B] = color.map((v) => v / 255);
      const floatColor = { R, G, B, A: 1 };
      conversionFn(floatColor);
      return [
      floatToU8(floatColor.R),
      floatToU8(floatColor.G),
      floatToU8(floatColor.B),
      floatToU8(floatColor.A)];

    }).
    flat()
  );

  const subrectOrigin = [0, 0, 0];
  const subrectSize = [width, height, 1];
  const areaDesc = {
    bytesPerRow: width * 4,
    rowsPerImage: height,
    subrectOrigin,
    subrectSize
  };

  const format = 'rgba8unorm';
  const actTexelView = TexelView.fromTextureDataByReference(format, convertedData, areaDesc);
  const expTexelView = TexelView.fromTextureDataByReference(format, expectedData, areaDesc);

  const failedPixelsMessage = findFailedPixels(
    format,
    { x: 0, y: 0, z: 0 },
    { width, height, depthOrArrayLayers: 1 },
    { actTexelView, expTexelView },
    { maxDiffULPsForNormFormat: 0 }
  );

  if (failedPixelsMessage !== undefined) {
    const msg = 'Color space conversion had unexpected results:\n' + failedPixelsMessage;
    t.expectOK(
      new ErrorWithExtra(msg, () => ({
        expTexelView,
        actTexelView
      }))
    );
  }
});