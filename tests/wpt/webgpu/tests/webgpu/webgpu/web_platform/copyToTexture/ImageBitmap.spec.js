/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
copyExternalImageToTexture from ImageBitmaps created from various sources.

TODO: Test ImageBitmap generated from all possible ImageBitmapSource, relevant ImageBitmapOptions
    (https://html.spec.whatwg.org/multipage/imagebitmap-and-animations.html#images-2)
    and various source filetypes and metadata (weird dimensions, EXIF orientations, video rotations
    and visible/crop rectangles, etc. (In theory these things are handled inside createImageBitmap,
    but in theory could affect the internal representation of the ImageBitmap.)

TODO: Test zero-sized copies from all sources (just make sure params cover it) (e.g. 0x0, 0x4, 4x0).
`;import { makeTestGroup } from '../../../common/framework/test_group.js';
import { kTextureFormatInfo, kValidTextureFormatsForCopyE2T } from '../../format_info.js';
import { TextureUploadingUtils, kCopySubrectInfo } from '../../util/copy_to_texture.js';

import { kTestColorsAll, kTestColorsOpaque, makeTestColorsTexelView } from './util.js';

export const g = makeTestGroup(TextureUploadingUtils);

g.test('from_ImageData').
desc(
  `
  Test ImageBitmap generated from ImageData can be copied to WebGPU
  texture correctly. These imageBitmaps are highly possible living
  in CPU back resource.

  It generates pixels in ImageData one by one based on a color list:
  [Red, Green, Blue, Black, White, SemitransparentWhite].

  Then call copyExternalImageToTexture() to do a full copy to the 0 mipLevel
  of dst texture, and read the contents out to compare with the ImageBitmap contents.

  Do premultiply alpha during copy if 'premultipliedAlpha' in 'GPUImageCopyTextureTagged'
  is set to 'true' and do unpremultiply alpha if it is set to 'false'.

  If 'flipY' in 'GPUImageCopyExternalImage' is set to 'true', copy will ensure the result
  is flipped.

  The tests covers:
  - Valid dstFormat of copyExternalImageToTexture()
  - Valid source image alphaMode
  - Valid dest alphaMode
  - Valid 'flipY' config in 'GPUImageCopyExternalImage' (named 'srcFlipYInCopy' in cases)

  And the expected results are all passed.
  `
).
params((u) =>
u.
combine('alpha', ['none', 'premultiply']).
combine('orientation', ['none', 'flipY']).
combine('colorSpaceConversion', ['none', 'default']).
combine('srcFlipYInCopy', [true, false]).
combine('dstFormat', kValidTextureFormatsForCopyE2T).
combine('dstPremultiplied', [true, false]).
beginSubcases().
combine('width', [1, 2, 4, 15, 255, 256]).
combine('height', [1, 2, 4, 15, 255, 256])
).
beforeAllSubcases((t) => {
  t.skipIfTextureFormatNotSupported(t.params.dstFormat);
}).
fn(async (t) => {
  const {
    width,
    height,
    alpha,
    orientation,
    colorSpaceConversion,
    dstFormat,
    dstPremultiplied,
    srcFlipYInCopy
  } = t.params;

  const testColors = kTestColorsAll;

  // Generate correct expected values
  const texelViewSource = makeTestColorsTexelView({
    testColors,
    format: 'rgba8unorm', // ImageData is always in rgba8unorm format.
    width,
    height,
    flipY: false,
    premultiplied: false
  });
  const imageData = new ImageData(width, height);
  texelViewSource.writeTextureData(imageData.data, {
    bytesPerRow: width * 4,
    rowsPerImage: height,
    subrectOrigin: [0, 0],
    subrectSize: { width, height }
  });

  const imageBitmap = await createImageBitmap(imageData, {
    premultiplyAlpha: alpha,
    imageOrientation: orientation,
    colorSpaceConversion
  });

  const dst = t.createTextureTracked({
    size: { width, height },
    format: dstFormat,
    usage:
    GPUTextureUsage.COPY_DST | GPUTextureUsage.COPY_SRC | GPUTextureUsage.RENDER_ATTACHMENT
  });

  const expFormat = kTextureFormatInfo[dstFormat].baseFormat ?? dstFormat;
  const flipSrcBeforeCopy = orientation === 'flipY';
  const texelViewExpected = t.getExpectedDstPixelsFromSrcPixels({
    srcPixels: imageData.data,
    srcOrigin: [0, 0],
    srcSize: [width, height],
    dstOrigin: [0, 0],
    dstSize: [width, height],
    subRectSize: [width, height],
    format: expFormat,
    flipSrcBeforeCopy,
    srcDoFlipYDuringCopy: srcFlipYInCopy,
    conversion: {
      srcPremultiplied: false,
      dstPremultiplied
    }
  });

  t.doTestAndCheckResult(
    { source: imageBitmap, origin: { x: 0, y: 0 }, flipY: srcFlipYInCopy },
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

g.test('from_canvas').
desc(
  `
  Test ImageBitmap generated from canvas/offscreenCanvas can be copied to WebGPU
  texture correctly. These imageBitmaps are highly possible living in GPU back resource.

  It generates pixels in ImageData one by one based on a color list:
  [Red, Green, Blue, Black, White].

  Then call copyExternalImageToTexture() to do a full copy to the 0 mipLevel
  of dst texture, and read the contents out to compare with the ImageBitmap contents.

  Do premultiply alpha during copy if 'premultipliedAlpha' in 'GPUImageCopyTextureTagged'
  is set to 'true' and do unpremultiply alpha if it is set to 'false'.

  If 'flipY' in 'GPUImageCopyExternalImage' is set to 'true', copy will ensure the result
  is flipped.

  The tests covers:
  - Valid 2D canvas
  - Valid dstFormat of copyExternalImageToTexture()
  - Valid source image alphaMode
  - Valid dest alphaMode
  - Valid 'flipY' config in 'GPUImageCopyExternalImage' (named 'srcFlipYInCopy' in cases)

  And the expected results are all passed.
  `
).
params((u) =>
u.
combine('orientation', ['none', 'flipY']).
combine('colorSpaceConversion', ['none', 'default']).
combine('srcFlipYInCopy', [true, false]).
combine('dstFormat', kValidTextureFormatsForCopyE2T).
combine('dstPremultiplied', [true, false]).
beginSubcases().
combine('width', [1, 2, 4, 15, 255, 256]).
combine('height', [1, 2, 4, 15, 255, 256])
).
beforeAllSubcases((t) => {
  t.skipIfTextureFormatNotSupported(t.params.dstFormat);
}).
fn(async (t) => {
  const {
    width,
    height,
    orientation,
    colorSpaceConversion,
    dstFormat,
    dstPremultiplied,
    srcFlipYInCopy
  } = t.params;

  // CTS sometimes runs on worker threads, where document is not available.
  // In this case, OffscreenCanvas can be used instead of <canvas>.
  // But some browsers don't support OffscreenCanvas, and some don't
  // support '2d' contexts on OffscreenCanvas.
  // In this situation, the case will be skipped.
  let imageCanvas;
  if (typeof document !== 'undefined') {
    imageCanvas = document.createElement('canvas');
    imageCanvas.width = width;
    imageCanvas.height = height;
  } else if (typeof OffscreenCanvas === 'undefined') {
    t.skip('OffscreenCanvas is not supported');
    return;
  } else {
    imageCanvas = new OffscreenCanvas(width, height);
  }
  const imageCanvasContext = imageCanvas.getContext('2d');
  if (imageCanvasContext === null) {
    t.skip('OffscreenCanvas "2d" context not available');
    return;
  }

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

  // Use putImageData to prevent color space conversion.
  imageCanvasContext.putImageData(imageData, 0, 0);

  // MAINTENANCE_TODO: Workaround for @types/offscreencanvas missing an overload of
  // `createImageBitmap` that takes `ImageBitmapOptions`.
  const imageBitmap = await createImageBitmap(imageCanvas, {
    premultiplyAlpha: 'premultiply',
    imageOrientation: orientation,
    colorSpaceConversion
  });

  const dst = t.createTextureTracked({
    size: { width, height },
    format: dstFormat,
    usage:
    GPUTextureUsage.COPY_DST | GPUTextureUsage.COPY_SRC | GPUTextureUsage.RENDER_ATTACHMENT
  });

  const expFormat = kTextureFormatInfo[dstFormat].baseFormat ?? dstFormat;
  const flipSrcBeforeCopy = orientation === 'flipY';
  const texelViewExpected = t.getExpectedDstPixelsFromSrcPixels({
    srcPixels: imageData.data,
    srcOrigin: [0, 0],
    srcSize: [width, height],
    dstOrigin: [0, 0],
    dstSize: [width, height],
    subRectSize: [width, height],
    format: expFormat,
    flipSrcBeforeCopy,
    srcDoFlipYDuringCopy: srcFlipYInCopy,
    conversion: {
      srcPremultiplied: false,
      dstPremultiplied
    }
  });

  t.doTestAndCheckResult(
    { source: imageBitmap, origin: { x: 0, y: 0 }, flipY: srcFlipYInCopy },
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

g.test('copy_subrect_from_ImageData').
desc(
  `
  Test ImageBitmap generated from ImageData can be copied to WebGPU
  texture correctly. These imageBitmaps are highly possible living in CPU back resource.

  It generates pixels in ImageData one by one based on a color list:
  [Red, Green, Blue, Black, White].

  Then call copyExternalImageToTexture() to do a subrect copy, based on a predefined copy
  rect info list, to the 0 mipLevel of dst texture, and read the contents out to compare
  with the ImageBitmap contents.

  Do premultiply alpha during copy if 'premultipliedAlpha' in 'GPUImageCopyTextureTagged'
  is set to 'true' and do unpremultiply alpha if it is set to 'false'.

  If 'flipY' in 'GPUImageCopyExternalImage' is set to 'true', copy will ensure the result
  is flipped, and origin is top-left consistantly.

  The tests covers:
  - Source WebGPU Canvas lives in the same GPUDevice or different GPUDevice as test
  - Valid dstFormat of copyExternalImageToTexture()
  - Valid source image alphaMode
  - Valid dest alphaMode
  - Valid 'flipY' config in 'GPUImageCopyExternalImage' (named 'srcFlipYInCopy' in cases)
  - Valid subrect copies.

  And the expected results are all passed.
  `
).
params((u) =>
u.
combine('alpha', ['none', 'premultiply']).
combine('orientation', ['none', 'flipY']).
combine('colorSpaceConversion', ['none', 'default']).
combine('srcFlipYInCopy', [true, false]).
combine('dstPremultiplied', [true, false]).
beginSubcases().
combine('copySubRectInfo', kCopySubrectInfo)
).
fn(async (t) => {
  const {
    copySubRectInfo,
    alpha,
    orientation,
    colorSpaceConversion,
    dstPremultiplied,
    srcFlipYInCopy
  } = t.params;

  const testColors = kTestColorsAll;
  const { srcOrigin, dstOrigin, srcSize, dstSize, copyExtent } = copySubRectInfo;
  const kColorFormat = 'rgba8unorm';

  // Generate correct expected values
  const texelViewSource = makeTestColorsTexelView({
    testColors,
    format: kColorFormat, // ImageData is always in rgba8unorm format.
    width: srcSize.width,
    height: srcSize.height,
    flipY: false,
    premultiplied: false
  });
  const imageData = new ImageData(srcSize.width, srcSize.height);
  texelViewSource.writeTextureData(imageData.data, {
    bytesPerRow: srcSize.width * 4,
    rowsPerImage: srcSize.height,
    subrectOrigin: [0, 0],
    subrectSize: srcSize
  });

  const imageBitmap = await createImageBitmap(imageData, {
    premultiplyAlpha: alpha,
    imageOrientation: orientation,
    colorSpaceConversion
  });

  const dst = t.createTextureTracked({
    size: dstSize,
    format: kColorFormat,
    usage:
    GPUTextureUsage.COPY_DST | GPUTextureUsage.COPY_SRC | GPUTextureUsage.RENDER_ATTACHMENT
  });

  const flipSrcBeforeCopy = orientation === 'flipY';
  const texelViewExpected = t.getExpectedDstPixelsFromSrcPixels({
    srcPixels: imageData.data,
    srcOrigin,
    srcSize,
    dstOrigin,
    dstSize,
    subRectSize: copyExtent,
    format: kColorFormat,
    flipSrcBeforeCopy,
    srcDoFlipYDuringCopy: srcFlipYInCopy,
    conversion: {
      srcPremultiplied: false,
      dstPremultiplied
    }
  });

  t.doTestAndCheckResult(
    { source: imageBitmap, origin: srcOrigin, flipY: srcFlipYInCopy },
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

g.test('copy_subrect_from_2D_Canvas').
desc(
  `
  Test ImageBitmap generated from canvas/offscreenCanvas can be copied to WebGPU
  texture correctly. These imageBitmaps are highly possible living in GPU back resource.

  It generates pixels in ImageData one by one based on a color list:
  [Red, Green, Blue, Black, White].

  Then call copyExternalImageToTexture() to do a subrect copy, based on a predefined copy
  rect info list, to the 0 mipLevel of dst texture, and read the contents out to compare
  with the ImageBitmap contents.

  Do premultiply alpha during copy if 'premultipliedAlpha' in 'GPUImageCopyTextureTagged'
  is set to 'true' and do unpremultiply alpha if it is set to 'false'.

  If 'flipY' in 'GPUImageCopyExternalImage' is set to 'true', copy will ensure the result
  is flipped, and origin is top-left consistantly.

  The tests covers:
  - Source WebGPU Canvas lives in the same GPUDevice or different GPUDevice as test
  - Valid dstFormat of copyExternalImageToTexture()
  - Valid source image alphaMode
  - Valid dest alphaMode
  - Valid 'flipY' config in 'GPUImageCopyExternalImage' (named 'srcFlipYInCopy' in cases)
  - Valid subrect copies.

  And the expected results are all passed.
  `
).
params((u) =>
u.
combine('orientation', ['none', 'flipY']).
combine('colorSpaceConversion', ['none', 'default']).
combine('srcFlipYInCopy', [true, false]).
combine('dstPremultiplied', [true, false]).
beginSubcases().
combine('copySubRectInfo', kCopySubrectInfo)
).
fn(async (t) => {
  const { copySubRectInfo, orientation, colorSpaceConversion, dstPremultiplied, srcFlipYInCopy } =
  t.params;

  const { srcOrigin, dstOrigin, srcSize, dstSize, copyExtent } = copySubRectInfo;
  const kColorFormat = 'rgba8unorm';

  // CTS sometimes runs on worker threads, where document is not available.
  // In this case, OffscreenCanvas can be used instead of <canvas>.
  // But some browsers don't support OffscreenCanvas, and some don't
  // support '2d' contexts on OffscreenCanvas.
  // In this situation, the case will be skipped.
  let imageCanvas;
  if (typeof document !== 'undefined') {
    imageCanvas = document.createElement('canvas');
    imageCanvas.width = srcSize.width;
    imageCanvas.height = srcSize.height;
  } else if (typeof OffscreenCanvas === 'undefined') {
    t.skip('OffscreenCanvas is not supported');
    return;
  } else {
    imageCanvas = new OffscreenCanvas(srcSize.width, srcSize.height);
  }
  const imageCanvasContext = imageCanvas.getContext('2d');
  if (imageCanvasContext === null) {
    t.skip('OffscreenCanvas "2d" context not available');
    return;
  }

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

  // Use putImageData to prevent color space conversion.
  imageCanvasContext.putImageData(imageData, 0, 0);

  // MAINTENANCE_TODO: Workaround for @types/offscreencanvas missing an overload of
  // `createImageBitmap` that takes `ImageBitmapOptions`.
  const imageBitmap = await createImageBitmap(imageCanvas, {
    premultiplyAlpha: 'premultiply',
    imageOrientation: orientation,
    colorSpaceConversion
  });

  const dst = t.createTextureTracked({
    size: dstSize,
    format: kColorFormat,
    usage:
    GPUTextureUsage.COPY_DST | GPUTextureUsage.COPY_SRC | GPUTextureUsage.RENDER_ATTACHMENT
  });

  const flipSrcBeforeCopy = orientation === 'flipY';
  const texelViewExpected = t.getExpectedDstPixelsFromSrcPixels({
    srcPixels: imageData.data,
    srcOrigin,
    srcSize,
    dstOrigin,
    dstSize,
    subRectSize: copyExtent,
    format: kColorFormat,
    flipSrcBeforeCopy,
    srcDoFlipYDuringCopy: srcFlipYInCopy,
    conversion: {
      srcPremultiplied: false,
      dstPremultiplied
    }
  });

  t.doTestAndCheckResult(
    { source: imageBitmap, origin: srcOrigin, flipY: srcFlipYInCopy },
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