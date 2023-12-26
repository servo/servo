/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
copyExternalImageToTexture from HTMLImageElement source.
`;import { makeTestGroup } from '../../../common/framework/test_group.js';
import { raceWithRejectOnTimeout } from '../../../common/util/util.js';
import { kTextureFormatInfo, kValidTextureFormatsForCopyE2T } from '../../format_info.js';
import { CopyToTextureUtils, kCopySubrectInfo } from '../../util/copy_to_texture.js';

import { kTestColorsOpaque, makeTestColorsTexelView } from './util.js';

async function decodeImageFromCanvas(canvas) {
  const blobFromCanvas = new Promise((resolve) => {
    canvas.toBlob((blob) => resolve(blob));
  });
  const blob = await blobFromCanvas;
  const url = URL.createObjectURL(blob);
  const image = new Image(canvas.width, canvas.height);
  image.src = url;
  await raceWithRejectOnTimeout(image.decode(), 5000, 'decode image timeout');
  return image;
}

export const g = makeTestGroup(CopyToTextureUtils);

g.test('from_image').
desc(
  `
  Test HTMLImageElement can be copied to WebGPU texture correctly.
  These images are highly possible living in GPU back resource.

  It generates pixels in ImageData one by one based on a color list:
  [Red, Green, Blue, Black, White].

  Then call copyExternalImageToTexture() to do a full copy to the 0 mipLevel
  of dst texture, and read the contents out to compare with the HTMLImageElement contents.

  Do premultiply alpha during copy if 'premultipliedAlpha' in 'GPUImageCopyTextureTagged'
  is set to 'true' and do unpremultiply alpha if it is set to 'false'.

  If 'flipY' in 'GPUImageCopyExternalImage' is set to 'true', copy will ensure the result
  is flipped.

  The tests covers:
  - Valid 2D canvas
  - Valid dstColorFormat of copyExternalImageToTexture()
  - Valid source image alphaMode
  - Valid dest alphaMode
  - Valid 'flipY' config in 'GPUImageCopyExternalImage' (named 'srcDoFlipYDuringCopy' in cases)

  And the expected results are all passed.
  `
).
params((u) =>
u.
combine('srcDoFlipYDuringCopy', [true, false]).
combine('dstColorFormat', kValidTextureFormatsForCopyE2T).
combine('dstPremultiplied', [true, false]).
beginSubcases().
combine('width', [1, 2, 4, 15, 255, 256]).
combine('height', [1, 2, 4, 15, 255, 256])
).
beforeAllSubcases((t) => {
  t.skipIfTextureFormatNotSupported(t.params.dstColorFormat);
  if (typeof HTMLImageElement === 'undefined') t.skip('HTMLImageElement not available');
}).
fn(async (t) => {
  const { width, height, dstColorFormat, dstPremultiplied, srcDoFlipYDuringCopy } = t.params;

  const imageCanvas = document.createElement('canvas');
  imageCanvas.width = width;
  imageCanvas.height = height;

  // Generate non-transparent pixel data to avoid canvas
  // different opt behaviour on putImageData()
  // from browsers.
  const texelViewSource = makeTestColorsTexelView({
    testColors: kTestColorsOpaque,
    format: 'rgba8unorm', // ImageData is always in rgba8unorm format.
    width,
    height,
    flipY: false,
    premultiplied: false
  });
  // Generate correct expected values
  const imageData = new ImageData(width, height);
  texelViewSource.writeTextureData(imageData.data, {
    bytesPerRow: width * 4,
    rowsPerImage: height,
    subrectOrigin: [0, 0],
    subrectSize: { width, height }
  });

  const imageCanvasContext = imageCanvas.getContext('2d');
  if (imageCanvasContext === null) {
    t.skip('canvas cannot get 2d context');
    return;
  }
  // Use putImageData to prevent color space conversion.
  imageCanvasContext.putImageData(imageData, 0, 0);

  const image = await decodeImageFromCanvas(imageCanvas);

  const dst = t.device.createTexture({
    size: { width, height },
    format: dstColorFormat,
    usage:
    GPUTextureUsage.COPY_DST | GPUTextureUsage.COPY_SRC | GPUTextureUsage.RENDER_ATTACHMENT
  });

  const expFormat = kTextureFormatInfo[dstColorFormat].baseFormat ?? dstColorFormat;
  const flipSrcBeforeCopy = false;
  const texelViewExpected = t.getExpectedDstPixelsFromSrcPixels({
    srcPixels: imageData.data,
    srcOrigin: [0, 0],
    srcSize: [width, height],
    dstOrigin: [0, 0],
    dstSize: [width, height],
    subRectSize: [width, height],
    format: expFormat,
    flipSrcBeforeCopy,
    srcDoFlipYDuringCopy,
    conversion: {
      srcPremultiplied: false,
      dstPremultiplied
    }
  });

  t.doTestAndCheckResult(
    {
      source: image,
      origin: { x: 0, y: 0 },
      flipY: srcDoFlipYDuringCopy
    },
    {
      texture: dst,
      origin: { x: 0, y: 0 },
      colorSpace: 'srgb',
      premultipliedAlpha: dstPremultiplied
    },
    texelViewExpected,
    { width, height, depthOrArrayLayers: 1 },
    // 1.0 and 0.6 are representable precisely by all formats except rgb10a2unorm, but
    // allow diffs of 1ULP since that's the generally-appropriate threshold.
    { maxDiffULPsForFloatFormat: 1, maxDiffULPsForNormFormat: 1 }
  );
});

g.test('copy_subrect_from_2D_Canvas').
desc(
  `
  Test HTMLImageElement can be copied to WebGPU texture correctly.
  These images are highly possible living in GPU back resource.

  It generates pixels in ImageData one by one based on a color list:
  [Red, Green, Blue, Black, White].

  Then call copyExternalImageToTexture() to do a subrect copy, based on a predefined copy
  rect info list, to the 0 mipLevel of dst texture, and read the contents out to compare
  with the HTMLImageElement contents.

  Do premultiply alpha during copy if 'premultipliedAlpha' in 'GPUImageCopyTextureTagged'
  is set to 'true' and do unpremultiply alpha if it is set to 'false'.

  If 'flipY' in 'GPUImageCopyExternalImage' is set to 'true', copy will ensure the result
  is flipped, and origin is top-left consistantly.

  The tests covers:
  - Source WebGPU Canvas lives in the same GPUDevice or different GPUDevice as test
  - Valid dstColorFormat of copyExternalImageToTexture()
  - Valid source image alphaMode
  - Valid dest alphaMode
  - Valid 'flipY' config in 'GPUImageCopyExternalImage' (named 'srcDoFlipYDuringCopy' in cases)
  - Valid subrect copies.

  And the expected results are all passed.
  `
).
params((u) =>
u.
combine('srcDoFlipYDuringCopy', [true, false]).
combine('dstPremultiplied', [true, false]).
beginSubcases().
combine('copySubRectInfo', kCopySubrectInfo)
).
beforeAllSubcases((t) => {
  if (typeof HTMLImageElement === 'undefined') t.skip('HTMLImageElement not available');
}).
fn(async (t) => {
  const { copySubRectInfo, dstPremultiplied, srcDoFlipYDuringCopy } = t.params;

  const { srcOrigin, dstOrigin, srcSize, dstSize, copyExtent } = copySubRectInfo;
  const kColorFormat = 'rgba8unorm';

  const imageCanvas = document.createElement('canvas');
  imageCanvas.width = srcSize.width;
  imageCanvas.height = srcSize.height;

  // Generate non-transparent pixel data to avoid canvas
  // different opt behaviour on putImageData()
  // from browsers.
  const texelViewSource = makeTestColorsTexelView({
    testColors: kTestColorsOpaque,
    format: 'rgba8unorm', // ImageData is always in rgba8unorm format.
    width: srcSize.width,
    height: srcSize.height,
    flipY: false,
    premultiplied: false
  });
  // Generate correct expected values
  const imageData = new ImageData(srcSize.width, srcSize.height);
  texelViewSource.writeTextureData(imageData.data, {
    bytesPerRow: srcSize.width * 4,
    rowsPerImage: srcSize.height,
    subrectOrigin: [0, 0],
    subrectSize: srcSize
  });

  const imageCanvasContext = imageCanvas.getContext('2d');
  if (imageCanvasContext === null) {
    t.skip('canvas cannot get 2d context');
    return;
  }
  // Use putImageData to prevent color space conversion.
  imageCanvasContext.putImageData(imageData, 0, 0);

  const image = await decodeImageFromCanvas(imageCanvas);

  const dst = t.device.createTexture({
    size: dstSize,
    format: kColorFormat,
    usage:
    GPUTextureUsage.COPY_DST | GPUTextureUsage.COPY_SRC | GPUTextureUsage.RENDER_ATTACHMENT
  });

  const flipSrcBeforeCopy = false;
  const texelViewExpected = t.getExpectedDstPixelsFromSrcPixels({
    srcPixels: imageData.data,
    srcOrigin,
    srcSize,
    dstOrigin,
    dstSize,
    subRectSize: copyExtent,
    format: kColorFormat,
    flipSrcBeforeCopy,
    srcDoFlipYDuringCopy,
    conversion: {
      srcPremultiplied: false,
      dstPremultiplied
    }
  });

  t.doTestAndCheckResult(
    {
      source: image,
      origin: srcOrigin,
      flipY: srcDoFlipYDuringCopy
    },
    {
      texture: dst,
      origin: dstOrigin,
      colorSpace: 'srgb',
      premultipliedAlpha: dstPremultiplied
    },
    texelViewExpected,
    copyExtent,
    // 1.0 and 0.6 are representable precisely by all formats except rgb10a2unorm, but
    // allow diffs of 1ULP since that's the generally-appropriate threshold.
    { maxDiffULPsForFloatFormat: 1, maxDiffULPsForNormFormat: 1 }
  );
});