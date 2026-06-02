/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
copyExternalImageToTexture from ImageFiles like *.png, *.jpg source.
`;import { makeTestGroup } from '../../../common/framework/test_group.js';
import * as ttu from '../../texture_test_utils.js';
import { TextureUploadingUtils } from '../../util/copy_to_texture.js';
import {
  convertToUnorm8,
  getSourceFromEXIFImageFile,
  kImageNames,
  kImageInfo,
  kImageExpectedColors,
  kObjectTypeFromFiles,
  kEXIFImageNames,
  kEXIFImageInfo,
  loadImageFileAndRun } from
'../util.js';

export const g = makeTestGroup(TextureUploadingUtils);

g.test('from_orientation_metadata_file').
desc(
  `
    Test HTMLImageElements with rotation metadata can be copied to WebGPU texture correctly.

    It creates an ImageBitmap or HTMLImageElement using images in the 'resources' folder.

    Then call copyExternalImageToTexture() to do a full copy to the 0 mipLevel
    of dst texture, and read one pixel out to compare with the manually documented expected color.

    If 'flipY' in 'GPUCopyExternalImageSourceInfo' is set to 'true', copy will ensure the result
    is flipped.

    The tests covers:
    - Image with rotation metadata
    - Valid 'flipY' config in 'GPUCopyExternalImageSourceInfo' (named 'srcDoFlipYDuringCopy' in cases)
    - TODO: partial copy tests should be added
    - TODO: all valid dstColorFormat tests should be added.
    - TODO(#4108): Make this work in service workers (see GetSourceFromImageFile)
  `
).
params((u) =>
u //
.combine('imageName', kEXIFImageNames).
combine('objectTypeFromFile', kObjectTypeFromFiles).
combine('srcDoFlipYDuringCopy', [true, false])
).
fn(async (t) => {
  const { imageName, objectTypeFromFile, srcDoFlipYDuringCopy } = t.params;
  const kColorFormat = 'rgba8unorm';

  // Load image file.
  const source = await getSourceFromEXIFImageFile(t, imageName, objectTypeFromFile);
  const width = source.width;
  const height = source.height;

  const dstTexture = t.createTextureTracked({
    size: { width, height },
    format: kColorFormat,
    usage:
    GPUTextureUsage.COPY_DST | GPUTextureUsage.COPY_SRC | GPUTextureUsage.RENDER_ATTACHMENT
  });

  t.device.queue.copyExternalImageToTexture(
    {
      source,
      flipY: srcDoFlipYDuringCopy
    },
    {
      texture: dstTexture
    },
    {
      width,
      height
    }
  );

  const expect = kEXIFImageInfo[imageName].display;
  const presentColors = kImageExpectedColors.srgb;

  if (srcDoFlipYDuringCopy) {
    ttu.expectSinglePixelComparisonsAreOkInTexture(t, { texture: dstTexture }, [
    // Flipped top-left.
    {
      coord: { x: width * 0.25, y: height * 0.25 },
      exp: convertToUnorm8(presentColors[expect.bottomLeftColor])
    },
    // Flipped top-right.
    {
      coord: { x: width * 0.75, y: height * 0.25 },
      exp: convertToUnorm8(presentColors[expect.bottomRightColor])
    },
    // Flipped bottom-left.
    {
      coord: { x: width * 0.25, y: height * 0.75 },
      exp: convertToUnorm8(presentColors[expect.topLeftColor])
    },
    // Flipped bottom-right.
    {
      coord: { x: width * 0.75, y: height * 0.75 },
      exp: convertToUnorm8(presentColors[expect.topRightColor])
    }]
    );
  } else {
    ttu.expectSinglePixelComparisonsAreOkInTexture(t, { texture: dstTexture }, [
    // Top-left.
    {
      coord: { x: width * 0.25, y: height * 0.25 },
      exp: convertToUnorm8(presentColors[expect.topLeftColor])
    },
    // Top-right.
    {
      coord: { x: width * 0.75, y: height * 0.25 },
      exp: convertToUnorm8(presentColors[expect.topRightColor])
    },
    // Bottom-left.
    {
      coord: { x: width * 0.25, y: height * 0.75 },
      exp: convertToUnorm8(presentColors[expect.bottomLeftColor])
    },
    // Bottom-right.
    {
      coord: { x: width * 0.75, y: height * 0.75 },
      exp: convertToUnorm8(presentColors[expect.bottomRightColor])
    }]
    );
  }
});

g.test('from_multiple_formats').
desc(
  `
    Test HTMLImageElements which loaded multiple image file formats. Including
    *.jpg, *.png, *.bmp, *.webp, *.avif, *.svg, *.ico and *.gif.

    It creates an HTMLImageElement using images in the 'resources' folder.

    Then call copyExternalImageToTexture() to do a full copy to the 0 mipLevel
    of dst texture, and read one pixel out to compare with the manually documented expected color.

    If 'flipY' in 'GPUCopyExternalImageSourceInfo' is set to 'true', copy will ensure the result
    is flipped.

    The tests covers:
    - Image with multiple image file format
    - Valid 'flipY' config in 'GPUCopyExternalImageSourceInfo' (named 'srcDoFlipYDuringCopy' in cases)
    - TODO: partial copy tests should be added
    - TODO: all valid dstColorFormat tests should be added.
    - TODO(#4108): Make this work in service workers (see GetSourceFromImageFile)
  `
).
params((u) =>
u //
.combine('imageName', kImageNames).
combine('srcDoFlipYDuringCopy', [true, false])
).
fn(async (t) => {
  const { imageName, srcDoFlipYDuringCopy } = t.params;
  const kColorFormat = 'rgba8unorm';
  await loadImageFileAndRun(t, imageName, (source) => {
    const width = source.width;
    const height = source.height;

    const dstTexture = t.createTextureTracked({
      size: { width, height },
      format: kColorFormat,
      usage:
      GPUTextureUsage.COPY_DST | GPUTextureUsage.COPY_SRC | GPUTextureUsage.RENDER_ATTACHMENT
    });

    t.device.queue.copyExternalImageToTexture(
      {
        source,
        flipY: srcDoFlipYDuringCopy
      },
      {
        texture: dstTexture
      },
      {
        width,
        height
      }
    );

    const expect = kImageInfo[imageName].display;
    const presentColors = kImageExpectedColors.srgb;

    if (srcDoFlipYDuringCopy) {
      ttu.expectSinglePixelComparisonsAreOkInTexture(t, { texture: dstTexture }, [
      // Flipped top-left.
      {
        coord: { x: width * 0.25, y: height * 0.25 },
        exp: convertToUnorm8(presentColors[expect.bottomLeftColor])
      },
      // Flipped top-right.
      {
        coord: { x: width * 0.75, y: height * 0.25 },
        exp: convertToUnorm8(presentColors[expect.bottomRightColor])
      },
      // Flipped bottom-left.
      {
        coord: { x: width * 0.25, y: height * 0.75 },
        exp: convertToUnorm8(presentColors[expect.topLeftColor])
      },
      // Flipped bottom-right.
      {
        coord: { x: width * 0.75, y: height * 0.75 },
        exp: convertToUnorm8(presentColors[expect.topRightColor])
      }]
      );
    } else {
      ttu.expectSinglePixelComparisonsAreOkInTexture(t, { texture: dstTexture }, [
      // Top-left.
      {
        coord: { x: width * 0.25, y: height * 0.25 },
        exp: convertToUnorm8(presentColors[expect.topLeftColor])
      },
      // Top-right.
      {
        coord: { x: width * 0.75, y: height * 0.25 },
        exp: convertToUnorm8(presentColors[expect.topRightColor])
      },
      // Bottom-left.
      {
        coord: { x: width * 0.25, y: height * 0.75 },
        exp: convertToUnorm8(presentColors[expect.bottomLeftColor])
      },
      // Bottom-right.
      {
        coord: { x: width * 0.75, y: height * 0.75 },
        exp: convertToUnorm8(presentColors[expect.bottomRightColor])
      }]
      );
    }
  });
});