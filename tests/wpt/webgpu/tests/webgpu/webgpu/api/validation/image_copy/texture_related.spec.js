/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `Texture related validation tests for B2T copy and T2B copy and writeTexture.`;
import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { assert } from '../../../../common/util/util.js';
import { kTextureDimensions, kTextureUsages } from '../../../capability_info.js';
import { GPUConst } from '../../../constants.js';
import {
  kColorTextureFormats,
  kSizedTextureFormats,
  kTextureFormatInfo,
  textureDimensionAndFormatCompatible,
} from '../../../format_info.js';
import { kResourceStates } from '../../../gpu_test.js';
import { align } from '../../../util/math.js';
import { virtualMipSize } from '../../../util/texture/base.js';
import { kImageCopyTypes } from '../../../util/texture/layout.js';

import {
  ImageCopyTest,
  texelBlockAlignmentTestExpanderForValueToCoordinate,
  formatCopyableWithMethod,
  getACopyableAspectWithMethod,
} from './image_copy.js';

export const g = makeTestGroup(ImageCopyTest);

g.test('valid')
  .desc(
    `
Test that the texture must be valid and not destroyed.
- for all copy methods
- for all texture states
- for various dimensions
`
  )
  .params(u =>
    u //
      .combine('method', kImageCopyTypes)
      .combine('textureState', kResourceStates)
      .combineWithParams([
        { dimension: '1d', size: [4, 1, 1] },
        { dimension: '2d', size: [4, 4, 1] },
        { dimension: '2d', size: [4, 4, 3] },
        { dimension: '3d', size: [4, 4, 3] },
      ])
  )
  .fn(t => {
    const { method, textureState, size, dimension } = t.params;

    const texture = t.createTextureWithState(textureState, {
      size,
      dimension,
      format: 'rgba8unorm',
      usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.COPY_DST,
    });

    const success = textureState === 'valid';
    const submit = textureState !== 'invalid';

    t.testRun(
      { texture },
      { bytesPerRow: 0 },
      { width: 0, height: 0, depthOrArrayLayers: 0 },
      { dataSize: 1, method, success, submit }
    );
  });

g.test('texture,device_mismatch')
  .desc('Tests the image copies cannot be called with a texture created from another device')
  .paramsSubcasesOnly(u =>
    u.combine('method', kImageCopyTypes).combine('mismatched', [true, false])
  )
  .beforeAllSubcases(t => {
    t.selectMismatchedDeviceOrSkipTestCase(undefined);
  })
  .fn(t => {
    const { method, mismatched } = t.params;
    const sourceDevice = mismatched ? t.mismatchedDevice : t.device;

    const texture = sourceDevice.createTexture({
      size: { width: 4, height: 4, depthOrArrayLayers: 1 },
      format: 'rgba8unorm',
      usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.COPY_DST,
    });

    t.testRun(
      { texture },
      { bytesPerRow: 0 },
      { width: 0, height: 0, depthOrArrayLayers: 0 },
      { dataSize: 1, method, success: !mismatched }
    );
  });

g.test('usage')
  .desc(
    `
The texture must have the appropriate COPY_SRC/COPY_DST usage.
- for various copy methods
- for various dimensions
- for various usages
`
  )
  .params(u =>
    u
      .combine('method', kImageCopyTypes)
      .combineWithParams([
        { dimension: '1d', size: [4, 1, 1] },
        { dimension: '2d', size: [4, 4, 1] },
        { dimension: '2d', size: [4, 4, 3] },
        { dimension: '3d', size: [4, 4, 3] },
      ])
      .beginSubcases()
      // If usage0 and usage1 are the same, the usage being test is a single usage. Otherwise, it's
      // a combined usage.
      .combine('usage0', kTextureUsages)
      .combine('usage1', kTextureUsages)
      // RENDER_ATTACHMENT is not valid with 1d and 3d textures.
      .unless(
        ({ usage0, usage1, dimension }) =>
          ((usage0 | usage1) & GPUConst.TextureUsage.RENDER_ATTACHMENT) !== 0 &&
          (dimension === '1d' || dimension === '3d')
      )
  )
  .fn(t => {
    const { usage0, usage1, method, size, dimension } = t.params;

    const usage = usage0 | usage1;
    const texture = t.device.createTexture({
      size,
      dimension,
      format: 'rgba8unorm',
      usage,
    });

    const success =
      method === 'CopyT2B'
        ? (usage & GPUTextureUsage.COPY_SRC) !== 0
        : (usage & GPUTextureUsage.COPY_DST) !== 0;

    t.testRun(
      { texture },
      { bytesPerRow: 0 },
      { width: 0, height: 0, depthOrArrayLayers: 0 },
      { dataSize: 1, method, success }
    );
  });

g.test('sample_count')
  .desc(
    `
Test that multisampled textures cannot be copied.
- for various copy methods
- multisampled or not

Note: we don't test 1D, 2D array and 3D textures because multisample is not supported them.
`
  )
  .params(u =>
    u //
      .combine('method', kImageCopyTypes)
      .beginSubcases()
      .combine('sampleCount', [1, 4])
  )
  .fn(t => {
    const { sampleCount, method } = t.params;

    const texture = t.device.createTexture({
      size: { width: 4, height: 4, depthOrArrayLayers: 1 },
      sampleCount,
      format: 'rgba8unorm',
      usage:
        GPUTextureUsage.COPY_SRC |
        GPUTextureUsage.COPY_DST |
        GPUTextureUsage.TEXTURE_BINDING |
        GPUTextureUsage.RENDER_ATTACHMENT,
    });

    const success = sampleCount === 1;

    t.testRun(
      { texture },
      { bytesPerRow: 0 },
      { width: 0, height: 0, depthOrArrayLayers: 0 },
      { dataSize: 1, method, success }
    );
  });

g.test('mip_level')
  .desc(
    `
Test that the mipLevel of the copy must be in range of the texture.
- for various copy methods
- for various dimensions
- for several mipLevelCounts
- for several target/source mipLevels`
  )
  .params(u =>
    u
      .combine('method', kImageCopyTypes)
      .combineWithParams([
        { dimension: '1d', size: [32, 1, 1] },
        { dimension: '2d', size: [32, 32, 1] },
        { dimension: '2d', size: [32, 32, 3] },
        { dimension: '3d', size: [32, 32, 3] },
      ])
      .beginSubcases()
      .combine('mipLevelCount', [1, 3, 5])
      .unless(p => p.dimension === '1d' && p.mipLevelCount !== 1)
      .combine('mipLevel', [0, 1, 3, 4])
  )
  .fn(t => {
    const { mipLevelCount, mipLevel, method, size, dimension } = t.params;

    const texture = t.device.createTexture({
      size,
      dimension,
      mipLevelCount,
      format: 'rgba8unorm',
      usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.COPY_DST,
    });

    const success = mipLevel < mipLevelCount;

    t.testRun(
      { texture, mipLevel },
      { bytesPerRow: 0 },
      { width: 0, height: 0, depthOrArrayLayers: 0 },
      { dataSize: 1, method, success }
    );
  });

g.test('format')
  .desc(
    `
Test the copy must be a full subresource if the texture's format is depth/stencil format.
- for various copy methods
- for various dimensions
- for all sized formats
- for a couple target/source mipLevels
- for some modifier (or not) for the full copy size
`
  )
  .params(u =>
    u //
      .combine('method', kImageCopyTypes)
      .combineWithParams([
        { depthOrArrayLayers: 1, dimension: '1d' },
        { depthOrArrayLayers: 1, dimension: '2d' },
        { depthOrArrayLayers: 3, dimension: '2d' },
        { depthOrArrayLayers: 32, dimension: '3d' },
      ])
      .combine('format', kSizedTextureFormats)
      .filter(({ dimension, format }) => textureDimensionAndFormatCompatible(dimension, format))
      .filter(formatCopyableWithMethod)
      .beginSubcases()
      .combine('mipLevel', [0, 2])
      .unless(p => p.dimension === '1d' && p.mipLevel !== 0)
      .combine('copyWidthModifier', [0, -1])
      .combine('copyHeightModifier', [0, -1])
      // If the texture has multiple depth/array slices and it is not a 3D texture, which means it is an array texture,
      // depthModifier is not needed upon the third dimension. Because different layers are different subresources in
      // an array texture. Whether it is a full copy or non-full copy doesn't make sense across different subresources.
      // However, different depth slices on the same mip level are within the same subresource for a 3d texture. So we
      // need to examine depth dimension via copyDepthModifier to determine whether it is a full copy for a 3D texture.
      .expand('copyDepthModifier', ({ dimension: d }) => (d === '3d' ? [0, -1] : [0]))
  )
  .beforeAllSubcases(t => {
    const info = kTextureFormatInfo[t.params.format];
    t.skipIfTextureFormatNotSupported(t.params.format);
    t.selectDeviceOrSkipTestCase(info.feature);
  })
  .fn(t => {
    const {
      method,
      depthOrArrayLayers,
      dimension,
      format,
      mipLevel,
      copyWidthModifier,
      copyHeightModifier,
      copyDepthModifier,
    } = t.params;

    const info = kTextureFormatInfo[format];
    const size = { width: 32 * info.blockWidth, height: 32 * info.blockHeight, depthOrArrayLayers };
    if (dimension === '1d') {
      size.height = 1;
    }

    const texture = t.device.createTexture({
      size,
      dimension,
      format,
      mipLevelCount: dimension === '1d' ? 1 : 5,
      usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.COPY_DST,
    });

    let success = true;
    if (
      (info.depth || info.stencil) &&
      (copyWidthModifier !== 0 || copyHeightModifier !== 0 || copyDepthModifier !== 0)
    ) {
      success = false;
    }

    const levelSize = virtualMipSize(
      dimension,
      [size.width, size.height, size.depthOrArrayLayers],
      mipLevel
    );

    const copySize = [
      levelSize[0] + copyWidthModifier * info.blockWidth,
      levelSize[1] + copyHeightModifier * info.blockHeight,
      // Note that compressed format is not supported for 3D textures yet, so there is no info.blockDepth.
      levelSize[2] + copyDepthModifier,
    ];

    t.testRun(
      { texture, mipLevel, aspect: getACopyableAspectWithMethod({ format, method }) },
      { bytesPerRow: 512, rowsPerImage: 32 },
      copySize,
      {
        dataSize: 512 * 32 * 32,
        method,
        success,
      }
    );
  });

g.test('origin_alignment')
  .desc(
    `
Test that the texture copy origin must be aligned to the format's block size.
- for various copy methods
- for all color formats (depth stencil formats require a full copy)
- for X, Y and Z coordinates
- for various values for that coordinate depending on the block size
`
  )
  .params(u =>
    u
      .combine('method', kImageCopyTypes)
      // No need to test depth/stencil formats because its copy origin must be [0, 0, 0], which is already aligned with block size.
      .combine('format', kColorTextureFormats)
      .filter(formatCopyableWithMethod)
      .combineWithParams([
        { depthOrArrayLayers: 1, dimension: '1d' },
        { depthOrArrayLayers: 1, dimension: '2d' },
        { depthOrArrayLayers: 3, dimension: '2d' },
        { depthOrArrayLayers: 3, dimension: '3d' },
      ])
      .filter(({ dimension, format }) => textureDimensionAndFormatCompatible(dimension, format))
      .beginSubcases()
      .combine('coordinateToTest', ['x', 'y', 'z'])
      .unless(p => p.dimension === '1d' && p.coordinateToTest !== 'x')
      .expand('valueToCoordinate', texelBlockAlignmentTestExpanderForValueToCoordinate)
  )
  .beforeAllSubcases(t => {
    const info = kTextureFormatInfo[t.params.format];
    t.skipIfTextureFormatNotSupported(t.params.format);
    t.selectDeviceOrSkipTestCase(info.feature);
  })
  .fn(t => {
    const {
      valueToCoordinate,
      coordinateToTest,
      format,
      method,
      depthOrArrayLayers,
      dimension,
    } = t.params;
    const info = kTextureFormatInfo[format];
    const size = { width: 0, height: 0, depthOrArrayLayers };
    const origin = { x: 0, y: 0, z: 0 };
    let success = true;

    origin[coordinateToTest] = valueToCoordinate;
    switch (coordinateToTest) {
      case 'x': {
        success = origin.x % info.blockWidth === 0;
        break;
      }
      case 'y': {
        success = origin.y % info.blockHeight === 0;
        break;
      }
    }

    const texture = t.createAlignedTexture(format, size, origin, dimension);

    t.testRun({ texture, origin }, { bytesPerRow: 0, rowsPerImage: 0 }, size, {
      dataSize: 1,
      method,
      success,
    });
  });

g.test('size_alignment')
  .desc(
    `
Test that the copy size must be aligned to the texture's format's block size.
- for various copy methods
- for all formats (depth-stencil formats require a full copy)
- for all texture dimensions
- for the size's parameters to test (width / height / depth)
- for various values for that copy size parameters, depending on the block size
`
  )
  .params(u =>
    u
      .combine('method', kImageCopyTypes)
      // No need to test depth/stencil formats because its copy size must be subresource's size, which is already aligned with block size.
      .combine('format', kColorTextureFormats)
      .filter(formatCopyableWithMethod)
      .combine('dimension', kTextureDimensions)
      .filter(({ dimension, format }) => textureDimensionAndFormatCompatible(dimension, format))
      .beginSubcases()
      .combine('coordinateToTest', ['width', 'height', 'depthOrArrayLayers'])
      .unless(p => p.dimension === '1d' && p.coordinateToTest !== 'width')
      .expand('valueToCoordinate', texelBlockAlignmentTestExpanderForValueToCoordinate)
  )
  .beforeAllSubcases(t => {
    const info = kTextureFormatInfo[t.params.format];
    t.skipIfTextureFormatNotSupported(t.params.format);
    t.selectDeviceOrSkipTestCase(info.feature);
  })
  .fn(t => {
    const { valueToCoordinate, coordinateToTest, dimension, format, method } = t.params;
    const info = kTextureFormatInfo[format];
    const size = { width: 0, height: 0, depthOrArrayLayers: 0 };
    const origin = { x: 0, y: 0, z: 0 };
    let success = true;

    size[coordinateToTest] = valueToCoordinate;
    switch (coordinateToTest) {
      case 'width': {
        success = size.width % info.blockWidth === 0;
        break;
      }
      case 'height': {
        success = size.height % info.blockHeight === 0;
        break;
      }
    }

    const texture = t.createAlignedTexture(format, size, origin, dimension);

    const bytesPerRow = align(
      Math.max(1, Math.ceil(size.width / info.blockWidth)) * info.bytesPerBlock,
      256
    );

    const rowsPerImage = Math.ceil(size.height / info.blockHeight);
    t.testRun({ texture, origin }, { bytesPerRow, rowsPerImage }, size, {
      dataSize: 1,
      method,
      success,
    });
  });

g.test('copy_rectangle')
  .desc(
    `
Test that the max corner of the copy rectangle (origin+copySize) must be inside the texture.
- for various copy methods
- for all dimensions
- for the X, Y and Z dimensions
- for various origin and copy size values (and texture sizes)
- for various mip levels
`
  )
  .params(u =>
    u
      .combine('method', kImageCopyTypes)
      .combine('dimension', kTextureDimensions)
      .beginSubcases()
      .combine('originValue', [7, 8])
      .combine('copySizeValue', [7, 8])
      .combine('textureSizeValue', [14, 15])
      .combine('mipLevel', [0, 2])
      .combine('coordinateToTest', [0, 1, 2])
      .unless(p => p.dimension === '1d' && (p.coordinateToTest !== 0 || p.mipLevel !== 0))
  )
  .fn(t => {
    const {
      originValue,
      copySizeValue,
      textureSizeValue,
      mipLevel,
      coordinateToTest,
      method,
      dimension,
    } = t.params;
    const format = 'rgba8unorm';
    const info = kTextureFormatInfo[format];

    const origin = [0, 0, 0];
    const copySize = [0, 0, 0];
    const textureSize = { width: 16 << mipLevel, height: 16 << mipLevel, depthOrArrayLayers: 16 };
    if (dimension === '1d') {
      textureSize.height = 1;
      textureSize.depthOrArrayLayers = 1;
    }
    const success = originValue + copySizeValue <= textureSizeValue;

    origin[coordinateToTest] = originValue;
    copySize[coordinateToTest] = copySizeValue;
    switch (coordinateToTest) {
      case 0: {
        textureSize.width = textureSizeValue << mipLevel;
        break;
      }
      case 1: {
        textureSize.height = textureSizeValue << mipLevel;
        break;
      }
      case 2: {
        textureSize.depthOrArrayLayers =
          dimension === '3d' ? textureSizeValue << mipLevel : textureSizeValue;
        break;
      }
    }

    const texture = t.device.createTexture({
      size: textureSize,
      dimension,
      mipLevelCount: dimension === '1d' ? 1 : 3,
      format,
      usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.COPY_DST,
    });

    assert(copySize[0] % info.blockWidth === 0);
    const bytesPerRow = align(copySize[0] / info.blockWidth, 256);
    assert(copySize[1] % info.blockHeight === 0);
    const rowsPerImage = copySize[1] / info.blockHeight;
    t.testRun({ texture, origin, mipLevel }, { bytesPerRow, rowsPerImage }, copySize, {
      dataSize: 1,
      method,
      success,
    });
  });
