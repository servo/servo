/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = '';
import { params, poptions } from '../../../../common/framework/params_builder.js';
import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { kSizedTextureFormats, kSizedTextureFormatInfo } from '../../../capability_info.js';

import {
  CopyBetweenLinearDataAndTextureTest,
  kAllTestMethods,
  texelBlockAlignmentTestExpanderForValueToCoordinate,
  formatCopyableWithMethod,
} from './copyBetweenLinearDataAndTexture.js';

export const g = makeTestGroup(CopyBetweenLinearDataAndTextureTest);

g.test('texture_must_be_valid')
  .params(
    params()
      .combine(poptions('method', kAllTestMethods))
      .combine(poptions('textureState', ['valid', 'destroyed', 'error']))
  )
  .fn(async t => {
    const { method, textureState } = t.params;

    // A valid texture.
    let texture = t.device.createTexture({
      size: { width: 4, height: 4, depth: 1 },
      format: 'rgba8unorm',
      usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.COPY_DST,
    });

    switch (textureState) {
      case 'destroyed': {
        texture.destroy();
        break;
      }
      case 'error': {
        texture = t.getErrorTexture();
        break;
      }
    }

    const success = textureState === 'valid';
    const submit = textureState === 'destroyed';

    t.testRun(
      { texture },
      { bytesPerRow: 0 },
      { width: 0, height: 0, depth: 0 },
      { dataSize: 1, method, success, submit }
    );
  });

g.test('texture_usage_must_be_valid')
  .params(
    params()
      .combine(poptions('method', kAllTestMethods))
      .combine(
        poptions('usage', [
          GPUTextureUsage.COPY_SRC | GPUTextureUsage.SAMPLED,
          GPUTextureUsage.COPY_DST | GPUTextureUsage.SAMPLED,
          GPUTextureUsage.COPY_SRC | GPUTextureUsage.COPY_DST,
        ])
      )
  )
  .fn(async t => {
    const { usage, method } = t.params;

    const texture = t.device.createTexture({
      size: { width: 4, height: 4, depth: 1 },
      format: 'rgba8unorm',
      usage,
    });

    const success =
      method === 'CopyTextureToBuffer'
        ? (usage & GPUTextureUsage.COPY_SRC) !== 0
        : (usage & GPUTextureUsage.COPY_DST) !== 0;

    t.testRun(
      { texture },
      { bytesPerRow: 0 },
      { width: 0, height: 0, depth: 0 },
      { dataSize: 1, method, success }
    );
  });

g.test('sample_count_must_be_1')
  .params(
    params()
      .combine(poptions('method', kAllTestMethods))
      .combine(poptions('sampleCount', [1, 4]))
  )
  .fn(async t => {
    const { sampleCount, method } = t.params;

    const texture = t.device.createTexture({
      size: { width: 4, height: 4, depth: 1 },
      sampleCount,
      format: 'rgba8unorm',
      usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.COPY_DST | GPUTextureUsage.SAMPLED,
    });

    const success = sampleCount === 1;

    t.testRun(
      { texture },
      { bytesPerRow: 0 },
      { width: 0, height: 0, depth: 0 },
      { dataSize: 1, method, success }
    );
  });

g.test('mip_level_must_be_in_range')
  .params(
    params()
      .combine(poptions('method', kAllTestMethods))
      .combine(poptions('mipLevelCount', [3, 5]))
      .combine(poptions('mipLevel', [3, 4]))
  )
  .fn(async t => {
    const { mipLevelCount, mipLevel, method } = t.params;

    const texture = t.device.createTexture({
      size: { width: 32, height: 32, depth: 1 },
      mipLevelCount,
      format: 'rgba8unorm',
      usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.COPY_DST,
    });

    const success = mipLevel < mipLevelCount;

    t.testRun(
      { texture, mipLevel },
      { bytesPerRow: 0 },
      { width: 0, height: 0, depth: 0 },
      { dataSize: 1, method, success }
    );
  });

g.test('texel_block_alignments_on_origin')
  .params(
    params()
      .combine(poptions('method', kAllTestMethods))
      .combine(poptions('coordinateToTest', ['x', 'y', 'z']))
      .combine(poptions('format', kSizedTextureFormats))
      .filter(formatCopyableWithMethod)
      .expand(texelBlockAlignmentTestExpanderForValueToCoordinate)
  )
  .fn(async t => {
    const { valueToCoordinate, coordinateToTest, format, method } = t.params;
    const info = kSizedTextureFormatInfo[format];

    const origin = { x: 0, y: 0, z: 0 };
    const size = { width: 0, height: 0, depth: 0 };
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

    const texture = t.createAlignedTexture(format, size, origin);

    t.testRun({ texture, origin }, { bytesPerRow: 0 }, size, {
      dataSize: 1,
      method,
      success,
    });
  });

g.test('1d_texture')
  .params(
    params()
      .combine(poptions('method', kAllTestMethods))
      .combine(poptions('width', [0, 1]))
      .combine([
        { height: 1, depth: 1 },
        { height: 1, depth: 0 },
        { height: 1, depth: 2 },
        { height: 0, depth: 1 },
        { height: 2, depth: 1 },
      ])
  )
  .fn(async t => {
    const { method, width, height, depth } = t.params;
    const size = { width, height, depth };

    const texture = t.device.createTexture({
      size: { width: 2, height: 1, depth: 1 },
      dimension: '1d',
      format: 'rgba8unorm',
      usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.COPY_DST,
    });

    // For 1d textures we require copyHeight and copyDepth to be 1,
    // copyHeight or copyDepth being 0 should cause a validation error.
    const success = size.height === 1 && size.depth === 1;

    t.testRun({ texture }, { bytesPerRow: 256, rowsPerImage: 4 }, size, {
      dataSize: 16,
      method,
      success,
    });
  });

g.test('texel_block_alignments_on_size')
  .params(
    params()
      .combine(poptions('method', kAllTestMethods))
      .combine(poptions('coordinateToTest', ['width', 'height', 'depth']))
      .combine(poptions('format', kSizedTextureFormats))
      .filter(formatCopyableWithMethod)
      .expand(texelBlockAlignmentTestExpanderForValueToCoordinate)
  )
  .fn(async t => {
    const { valueToCoordinate, coordinateToTest, format, method } = t.params;
    const info = kSizedTextureFormatInfo[format];

    const origin = { x: 0, y: 0, z: 0 };
    const size = { width: 0, height: 0, depth: 0 };
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

    const texture = t.createAlignedTexture(format, size, origin);

    t.testRun({ texture, origin }, { bytesPerRow: 0 }, size, {
      dataSize: 1,
      method,
      success,
    });
  });

g.test('texture_range_conditions')
  .params(
    params()
      .combine(poptions('method', kAllTestMethods))
      .combine(poptions('originValue', [7, 8]))
      .combine(poptions('copySizeValue', [7, 8]))
      .combine(poptions('textureSizeValue', [14, 15]))
      .combine(poptions('mipLevel', [0, 2]))
      .combine(poptions('coordinateToTest', [0, 1, 2]))
  )
  .fn(async t => {
    const {
      originValue,
      copySizeValue,
      textureSizeValue,
      mipLevel,
      coordinateToTest,
      method,
    } = t.params;

    const origin = [0, 0, 0];
    const copySize = [0, 0, 0];
    const textureSize = { width: 16 << mipLevel, height: 16 << mipLevel, depth: 16 };
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
        textureSize.depth = textureSizeValue;
        break;
      }
    }

    const texture = t.device.createTexture({
      size: textureSize,
      mipLevelCount: 3,
      format: 'rgba8unorm',
      usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.COPY_DST,
    });

    t.testRun({ texture, origin, mipLevel }, { bytesPerRow: 0 }, copySize, {
      dataSize: 1,
      method,
      success,
    });
  });
