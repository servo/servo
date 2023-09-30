/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
copyExternalImageToTexture from ImageData source.
`;
import { makeTestGroup } from '../../../common/framework/test_group.js';
import { kTextureFormatInfo, kValidTextureFormatsForCopyE2T } from '../../format_info.js';
import { CopyToTextureUtils, kCopySubrectInfo } from '../../util/copy_to_texture.js';

import { kTestColorsAll, makeTestColorsTexelView } from './util.js';

export const g = makeTestGroup(CopyToTextureUtils);

g.test('from_ImageData')
  .desc(
    `
  Test ImageData can be copied to WebGPU
  texture correctly. These imageDatas are highly possible living
  in CPU back resource.

  It generates pixels in ImageData one by one based on a color list:
  [Red, Green, Blue, Black, White, SemitransparentWhite].

  Then call copyExternalImageToTexture() to do a full copy to the 0 mipLevel
  of dst texture, and read the contents out to compare with the ImageData contents.

  Expect alpha to get premultiplied in the copy if, and only if, 'premultipliedAlpha'
  in 'GPUImageCopyTextureTagged' is set to 'true'.

  If 'flipY' in 'GPUImageCopyExternalImage' is set to 'true', copy will ensure the result
  is flipped.

  The tests covers:
  - Valid dstColorFormat of copyExternalImageToTexture()
  - Valid dest alphaMode
  - Valid 'flipY' config in 'GPUImageCopyExternalImage' (named 'srcDoFlipYDuringCopy' in cases)

  And the expected results are all passed.
  `
  )
  .params(u =>
    u
      .combine('srcDoFlipYDuringCopy', [true, false])
      .combine('dstColorFormat', kValidTextureFormatsForCopyE2T)
      .combine('dstPremultiplied', [true, false])
      .beginSubcases()
      .combine('width', [1, 2, 4, 15, 255, 256])
      .combine('height', [1, 2, 4, 15, 255, 256])
  )
  .beforeAllSubcases(t => {
    t.skipIfTextureFormatNotSupported(t.params.dstColorFormat);
  })
  .fn(t => {
    const { width, height, dstColorFormat, dstPremultiplied, srcDoFlipYDuringCopy } = t.params;

    const testColors = kTestColorsAll;

    // Generate correct expected values
    const texelViewSource = makeTestColorsTexelView({
      testColors,
      format: 'rgba8unorm', // ImageData is always in rgba8unorm format.
      width,
      height,
      flipY: false,
      premultiplied: false,
    });
    const imageData = new ImageData(width, height);
    texelViewSource.writeTextureData(imageData.data, {
      bytesPerRow: width * 4,
      rowsPerImage: height,
      subrectOrigin: [0, 0],
      subrectSize: { width, height },
    });

    const dst = t.device.createTexture({
      size: { width, height },
      format: dstColorFormat,
      usage:
        GPUTextureUsage.COPY_DST | GPUTextureUsage.COPY_SRC | GPUTextureUsage.RENDER_ATTACHMENT,
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
        dstPremultiplied,
      },
    });

    t.doTestAndCheckResult(
      {
        source: imageData,
        origin: { x: 0, y: 0 },
        flipY: srcDoFlipYDuringCopy,
      },
      {
        texture: dst,
        origin: { x: 0, y: 0 },
        colorSpace: 'srgb',
        premultipliedAlpha: dstPremultiplied,
      },
      texelViewExpected,
      { width, height, depthOrArrayLayers: 1 },
      // 1.0 and 0.6 are representable precisely by all formats except rgb10a2unorm, but
      // allow diffs of 1ULP since that's the generally-appropriate threshold.
      { maxDiffULPsForFloatFormat: 1, maxDiffULPsForNormFormat: 1 }
    );
  });

g.test('copy_subrect_from_ImageData')
  .desc(
    `
  Test ImageData can be copied to WebGPU
  texture correctly. These imageDatas are highly possible living in CPU back resource.

  It generates pixels in ImageData one by one based on a color list:
  [Red, Green, Blue, Black, White].

  Then call copyExternalImageToTexture() to do a subrect copy, based on a predefined copy
  rect info list, to the 0 mipLevel of dst texture, and read the contents out to compare
  with the ImageBitmap contents.

  Expect alpha to get premultiplied in the copy if, and only if, 'premultipliedAlpha'
  in 'GPUImageCopyTextureTagged' is set to 'true'.

  If 'flipY' in 'GPUImageCopyExternalImage' is set to 'true', copy will ensure the result
  is flipped, and origin is top-left consistantly.

  The tests covers:
  - Source WebGPU Canvas lives in the same GPUDevice or different GPUDevice as test
  - Valid dstColorFormat of copyExternalImageToTexture()
  - Valid dest alphaMode
  - Valid 'flipY' config in 'GPUImageCopyExternalImage' (named 'srcDoFlipYDuringCopy' in cases)
  - Valid subrect copies.

  And the expected results are all passed.
  `
  )
  .params(u =>
    u
      .combine('srcDoFlipYDuringCopy', [true, false])
      .combine('dstPremultiplied', [true, false])
      .beginSubcases()
      .combine('copySubRectInfo', kCopySubrectInfo)
  )
  .fn(t => {
    const { copySubRectInfo, dstPremultiplied, srcDoFlipYDuringCopy } = t.params;

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
      premultiplied: false,
    });
    const imageData = new ImageData(srcSize.width, srcSize.height);
    texelViewSource.writeTextureData(imageData.data, {
      bytesPerRow: srcSize.width * 4,
      rowsPerImage: srcSize.height,
      subrectOrigin: [0, 0],
      subrectSize: srcSize,
    });

    const dst = t.device.createTexture({
      size: dstSize,
      format: kColorFormat,
      usage:
        GPUTextureUsage.COPY_DST | GPUTextureUsage.COPY_SRC | GPUTextureUsage.RENDER_ATTACHMENT,
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
        dstPremultiplied,
      },
    });

    t.doTestAndCheckResult(
      {
        source: imageData,
        origin: srcOrigin,
        flipY: srcDoFlipYDuringCopy,
      },
      {
        texture: dst,
        origin: dstOrigin,
        colorSpace: 'srgb',
        premultipliedAlpha: dstPremultiplied,
      },
      texelViewExpected,
      copyExtent,
      // 1.0 and 0.6 are representable precisely by all formats except rgb10a2unorm, but
      // allow diffs of 1ULP since that's the generally-appropriate threshold.
      { maxDiffULPsForFloatFormat: 1, maxDiffULPsForNormFormat: 1 }
    );
  });
