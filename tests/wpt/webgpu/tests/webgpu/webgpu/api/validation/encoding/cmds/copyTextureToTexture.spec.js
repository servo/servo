/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
copyTextureToTexture tests.
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { kTextureUsages, kTextureDimensions } from '../../../../capability_info.js';
import {
  kTextureFormatInfo,
  kTextureFormats,
  kCompressedTextureFormats,
  kDepthStencilFormats,
  kFeaturesForFormats,
  filterFormatsByFeature,
  textureDimensionAndFormatCompatible } from
'../../../../format_info.js';
import { kResourceStates } from '../../../../gpu_test.js';
import { align, lcm } from '../../../../util/math.js';
import { ValidationTest } from '../../validation_test.js';

class F extends ValidationTest {
  TestCopyTextureToTexture(
  source,
  destination,
  copySize,
  expectation)
  {
    const commandEncoder = this.device.createCommandEncoder();
    commandEncoder.copyTextureToTexture(source, destination, copySize);

    if (expectation === 'FinishError') {
      this.expectValidationError(() => {
        commandEncoder.finish();
      });
    } else {
      const cmd = commandEncoder.finish();
      this.expectValidationError(() => {
        this.device.queue.submit([cmd]);
      }, expectation === 'SubmitError');
    }
  }

  GetPhysicalSubresourceSize(
  dimension,
  textureSize,
  format,
  mipLevel)
  {
    const virtualWidthAtLevel = Math.max(textureSize.width >> mipLevel, 1);
    const virtualHeightAtLevel = Math.max(textureSize.height >> mipLevel, 1);
    const physicalWidthAtLevel = align(virtualWidthAtLevel, kTextureFormatInfo[format].blockWidth);
    const physicalHeightAtLevel = align(
      virtualHeightAtLevel,
      kTextureFormatInfo[format].blockHeight
    );

    switch (dimension) {
      case '1d':
        return { width: physicalWidthAtLevel, height: 1, depthOrArrayLayers: 1 };
      case '2d':
        return {
          width: physicalWidthAtLevel,
          height: physicalHeightAtLevel,
          depthOrArrayLayers: textureSize.depthOrArrayLayers
        };
      case '3d':
        return {
          width: physicalWidthAtLevel,
          height: physicalHeightAtLevel,
          depthOrArrayLayers: Math.max(textureSize.depthOrArrayLayers >> mipLevel, 1)
        };
    }
  }
}

export const g = makeTestGroup(F);

g.test('copy_with_invalid_or_destroyed_texture').
desc('Test copyTextureToTexture is an error when one of the textures is invalid or destroyed.').
paramsSubcasesOnly((u) =>
u //
.combine('srcState', kResourceStates).
combine('dstState', kResourceStates)
).
fn((t) => {
  const { srcState, dstState } = t.params;

  const textureDesc = {
    size: { width: 4, height: 4, depthOrArrayLayers: 1 },
    format: 'rgba8unorm',
    usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.COPY_DST
  };

  const srcTexture = t.createTextureWithState(srcState, textureDesc);
  const dstTexture = t.createTextureWithState(dstState, textureDesc);

  const isSubmitSuccess = srcState === 'valid' && dstState === 'valid';
  const isFinishSuccess = srcState !== 'invalid' && dstState !== 'invalid';
  const expectation = isFinishSuccess ?
  isSubmitSuccess ?
  'Success' :
  'SubmitError' :
  'FinishError';

  t.TestCopyTextureToTexture(
    { texture: srcTexture },
    { texture: dstTexture },
    { width: 1, height: 1, depthOrArrayLayers: 1 },
    expectation
  );
});

g.test('texture,device_mismatch').
desc(
  'Tests copyTextureToTexture cannot be called with src texture or dst texture created from another device.'
).
paramsSubcasesOnly([
{ srcMismatched: false, dstMismatched: false }, // control case
{ srcMismatched: true, dstMismatched: false },
{ srcMismatched: false, dstMismatched: true }]
).
beforeAllSubcases((t) => {
  t.selectMismatchedDeviceOrSkipTestCase(undefined);
}).
fn((t) => {
  const { srcMismatched, dstMismatched } = t.params;

  const size = { width: 4, height: 4, depthOrArrayLayers: 1 };
  const format = 'rgba8unorm';

  const srcTextureDevice = srcMismatched ? t.mismatchedDevice : t.device;
  const srcTexture = srcTextureDevice.createTexture({
    size,
    format,
    usage: GPUTextureUsage.COPY_SRC
  });
  t.trackForCleanup(srcTexture);

  const dstTextureDevice = dstMismatched ? t.mismatchedDevice : t.device;
  const dstTexture = dstTextureDevice.createTexture({
    size,
    format,
    usage: GPUTextureUsage.COPY_DST
  });
  t.trackForCleanup(dstTexture);

  t.TestCopyTextureToTexture(
    { texture: srcTexture },
    { texture: dstTexture },
    { width: 1, height: 1, depthOrArrayLayers: 1 },
    srcMismatched || dstMismatched ? 'FinishError' : 'Success'
  );
});

g.test('mipmap_level').
desc(
  `
Test copyTextureToTexture must specify mipLevels that are in range.
- for various dimensions
- for various mip level count in the texture
- for various copy target mip level (in range and not in range)
`
).
params((u) =>
u //
.combine('dimension', kTextureDimensions).
beginSubcases().
combineWithParams([
{ srcLevelCount: 1, dstLevelCount: 1, srcCopyLevel: 0, dstCopyLevel: 0 },
{ srcLevelCount: 1, dstLevelCount: 1, srcCopyLevel: 1, dstCopyLevel: 0 },
{ srcLevelCount: 1, dstLevelCount: 1, srcCopyLevel: 0, dstCopyLevel: 1 },
{ srcLevelCount: 3, dstLevelCount: 3, srcCopyLevel: 0, dstCopyLevel: 0 },
{ srcLevelCount: 3, dstLevelCount: 3, srcCopyLevel: 2, dstCopyLevel: 0 },
{ srcLevelCount: 3, dstLevelCount: 3, srcCopyLevel: 3, dstCopyLevel: 0 },
{ srcLevelCount: 3, dstLevelCount: 3, srcCopyLevel: 0, dstCopyLevel: 2 },
{ srcLevelCount: 3, dstLevelCount: 3, srcCopyLevel: 0, dstCopyLevel: 3 }]
).
unless((p) => p.dimension === '1d' && (p.srcLevelCount !== 1 || p.dstLevelCount !== 1))
).

fn((t) => {
  const { srcLevelCount, dstLevelCount, srcCopyLevel, dstCopyLevel, dimension } = t.params;

  const srcTexture = t.device.createTexture({
    size: { width: 32, height: 1, depthOrArrayLayers: 1 },
    dimension,
    format: 'rgba8unorm',
    usage: GPUTextureUsage.COPY_SRC,
    mipLevelCount: srcLevelCount
  });
  const dstTexture = t.device.createTexture({
    size: { width: 32, height: 1, depthOrArrayLayers: 1 },
    dimension,
    format: 'rgba8unorm',
    usage: GPUTextureUsage.COPY_DST,
    mipLevelCount: dstLevelCount
  });

  const isSuccess = srcCopyLevel < srcLevelCount && dstCopyLevel < dstLevelCount;
  t.TestCopyTextureToTexture(
    { texture: srcTexture, mipLevel: srcCopyLevel },
    { texture: dstTexture, mipLevel: dstCopyLevel },
    { width: 1, height: 1, depthOrArrayLayers: 1 },
    isSuccess ? 'Success' : 'FinishError'
  );
});

g.test('texture_usage').
desc(
  `
Test that copyTextureToTexture source/destination need COPY_SRC/COPY_DST usages.
- for all possible source texture usages
- for all possible destination texture usages
`
).
paramsSubcasesOnly((u) =>
u //
.combine('srcUsage', kTextureUsages).
combine('dstUsage', kTextureUsages)
).
fn((t) => {
  const { srcUsage, dstUsage } = t.params;

  const srcTexture = t.device.createTexture({
    size: { width: 4, height: 4, depthOrArrayLayers: 1 },
    format: 'rgba8unorm',
    usage: srcUsage
  });
  const dstTexture = t.device.createTexture({
    size: { width: 4, height: 4, depthOrArrayLayers: 1 },
    format: 'rgba8unorm',
    usage: dstUsage
  });

  const isSuccess =
  srcUsage === GPUTextureUsage.COPY_SRC && dstUsage === GPUTextureUsage.COPY_DST;

  t.TestCopyTextureToTexture(
    { texture: srcTexture },
    { texture: dstTexture },
    { width: 1, height: 1, depthOrArrayLayers: 1 },
    isSuccess ? 'Success' : 'FinishError'
  );
});

g.test('sample_count').
desc(
  `
Test that textures in copyTextureToTexture must have the same sample count.
- for various source texture sample count
- for various destination texture sample count
`
).
paramsSubcasesOnly((u) =>
u //
.combine('srcSampleCount', [1, 4]).
combine('dstSampleCount', [1, 4])
).
fn((t) => {
  const { srcSampleCount, dstSampleCount } = t.params;

  const srcTexture = t.device.createTexture({
    size: { width: 4, height: 4, depthOrArrayLayers: 1 },
    format: 'rgba8unorm',
    usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.RENDER_ATTACHMENT,
    sampleCount: srcSampleCount
  });
  const dstTexture = t.device.createTexture({
    size: { width: 4, height: 4, depthOrArrayLayers: 1 },
    format: 'rgba8unorm',
    usage: GPUTextureUsage.COPY_DST | GPUTextureUsage.RENDER_ATTACHMENT,
    sampleCount: dstSampleCount
  });

  const isSuccess = srcSampleCount === dstSampleCount;
  t.TestCopyTextureToTexture(
    { texture: srcTexture },
    { texture: dstTexture },
    { width: 4, height: 4, depthOrArrayLayers: 1 },
    isSuccess ? 'Success' : 'FinishError'
  );
});

g.test('multisampled_copy_restrictions').
desc(
  `
Test that copyTextureToTexture of multisampled texture must copy a whole subresource to a whole subresource.
- for various origin for the source and destination of the copies.

Note: this is only tested for 2D textures as it is the only dimension compatible with multisampling.
TODO: Check the source and destination constraints separately.
`
).
paramsSubcasesOnly((u) =>
u //
.combine('srcCopyOrigin', [
{ x: 0, y: 0, z: 0 },
{ x: 1, y: 0, z: 0 },
{ x: 0, y: 1, z: 0 },
{ x: 1, y: 1, z: 0 }]
).
combine('dstCopyOrigin', [
{ x: 0, y: 0, z: 0 },
{ x: 1, y: 0, z: 0 },
{ x: 0, y: 1, z: 0 },
{ x: 1, y: 1, z: 0 }]
).
expand('copyWidth', (p) => [32 - Math.max(p.srcCopyOrigin.x, p.dstCopyOrigin.x), 16]).
expand('copyHeight', (p) => [16 - Math.max(p.srcCopyOrigin.y, p.dstCopyOrigin.y), 8])
).
fn((t) => {
  const { srcCopyOrigin, dstCopyOrigin, copyWidth, copyHeight } = t.params;

  const kWidth = 32;
  const kHeight = 16;

  // Currently we don't support multisampled 2D array textures and the mipmap level count of the
  // multisampled textures must be 1.
  const srcTexture = t.device.createTexture({
    size: { width: kWidth, height: kHeight, depthOrArrayLayers: 1 },
    format: 'rgba8unorm',
    usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.RENDER_ATTACHMENT,
    sampleCount: 4
  });
  const dstTexture = t.device.createTexture({
    size: { width: kWidth, height: kHeight, depthOrArrayLayers: 1 },
    format: 'rgba8unorm',
    usage: GPUTextureUsage.COPY_DST | GPUTextureUsage.RENDER_ATTACHMENT,
    sampleCount: 4
  });

  const isSuccess = copyWidth === kWidth && copyHeight === kHeight;
  t.TestCopyTextureToTexture(
    { texture: srcTexture, origin: srcCopyOrigin },
    { texture: dstTexture, origin: dstCopyOrigin },
    { width: copyWidth, height: copyHeight, depthOrArrayLayers: 1 },
    isSuccess ? 'Success' : 'FinishError'
  );
});

g.test('texture_format_compatibility').
desc(
  `
Test the formats of textures in copyTextureToTexture must be copy-compatible.
- for all source texture formats
- for all destination texture formats
`
).
params((u) =>
u.
combine('srcFormatFeature', kFeaturesForFormats).
combine('dstFormatFeature', kFeaturesForFormats).
beginSubcases().
expand('srcFormat', ({ srcFormatFeature }) =>
filterFormatsByFeature(srcFormatFeature, kTextureFormats)
).
expand('dstFormat', ({ dstFormatFeature }) =>
filterFormatsByFeature(dstFormatFeature, kTextureFormats)
)
).
beforeAllSubcases((t) => {
  const { srcFormatFeature, dstFormatFeature } = t.params;
  t.selectDeviceOrSkipTestCase([srcFormatFeature, dstFormatFeature]);
}).
fn((t) => {
  const { srcFormat, dstFormat } = t.params;

  t.skipIfTextureFormatNotSupported(srcFormat, dstFormat);
  t.skipIfCopyTextureToTextureNotSupportedForFormat(srcFormat, dstFormat);

  const srcFormatInfo = kTextureFormatInfo[srcFormat];
  const dstFormatInfo = kTextureFormatInfo[dstFormat];

  const textureSize = {
    width: lcm(srcFormatInfo.blockWidth, dstFormatInfo.blockWidth),
    height: lcm(srcFormatInfo.blockHeight, dstFormatInfo.blockHeight),
    depthOrArrayLayers: 1
  };

  const srcTexture = t.device.createTexture({
    size: textureSize,
    format: srcFormat,
    usage: GPUTextureUsage.COPY_SRC
  });

  const dstTexture = t.device.createTexture({
    size: textureSize,
    format: dstFormat,
    usage: GPUTextureUsage.COPY_DST
  });

  // Allow copy between compatible format textures.
  const srcBaseFormat = kTextureFormatInfo[srcFormat].baseFormat ?? srcFormat;
  const dstBaseFormat = kTextureFormatInfo[dstFormat].baseFormat ?? dstFormat;
  const isSuccess = srcBaseFormat === dstBaseFormat;

  t.TestCopyTextureToTexture(
    { texture: srcTexture },
    { texture: dstTexture },
    textureSize,
    isSuccess ? 'Success' : 'FinishError'
  );
});

g.test('depth_stencil_copy_restrictions').
desc(
  `
Test that depth textures subresources must be entirely copied in copyTextureToTexture
- for various depth-stencil formats
- for various copy origin and size offsets
- for various source and destination texture sizes
- for various source and destination mip levels

Note: this is only tested for 2D textures as it is the only dimension compatible with depth-stencil.
`
).
params((u) =>
u.
combine('format', kDepthStencilFormats).
beginSubcases().
combine('copyBoxOffsets', [
{ x: 0, y: 0, width: 0, height: 0 },
{ x: 1, y: 0, width: 0, height: 0 },
{ x: 0, y: 1, width: 0, height: 0 },
{ x: 0, y: 0, width: -1, height: 0 },
{ x: 0, y: 0, width: 0, height: -1 }]
).
combine('srcTextureSize', [
{ width: 64, height: 64, depthOrArrayLayers: 1 },
{ width: 64, height: 32, depthOrArrayLayers: 1 },
{ width: 32, height: 32, depthOrArrayLayers: 1 }]
).
combine('dstTextureSize', [
{ width: 64, height: 64, depthOrArrayLayers: 1 },
{ width: 64, height: 32, depthOrArrayLayers: 1 },
{ width: 32, height: 32, depthOrArrayLayers: 1 }]
).
combine('srcCopyLevel', [1, 2]).
combine('dstCopyLevel', [0, 1])
).
beforeAllSubcases((t) => {
  const { format } = t.params;
  t.selectDeviceOrSkipTestCase(kTextureFormatInfo[format].feature);
}).
fn((t) => {
  const { format, copyBoxOffsets, srcTextureSize, dstTextureSize, srcCopyLevel, dstCopyLevel } =
  t.params;
  const kMipLevelCount = 3;

  const srcTexture = t.device.createTexture({
    size: { width: srcTextureSize.width, height: srcTextureSize.height, depthOrArrayLayers: 1 },
    format,
    mipLevelCount: kMipLevelCount,
    usage: GPUTextureUsage.COPY_SRC
  });
  const dstTexture = t.device.createTexture({
    size: { width: dstTextureSize.width, height: dstTextureSize.height, depthOrArrayLayers: 1 },
    format,
    mipLevelCount: kMipLevelCount,
    usage: GPUTextureUsage.COPY_DST
  });

  const srcSizeAtLevel = t.GetPhysicalSubresourceSize('2d', srcTextureSize, format, srcCopyLevel);
  const dstSizeAtLevel = t.GetPhysicalSubresourceSize('2d', dstTextureSize, format, dstCopyLevel);

  const copyOrigin = { x: copyBoxOffsets.x, y: copyBoxOffsets.y, z: 0 };

  const copyWidth =
  Math.min(srcSizeAtLevel.width, dstSizeAtLevel.width) + copyBoxOffsets.width - copyOrigin.x;
  const copyHeight =
  Math.min(srcSizeAtLevel.height, dstSizeAtLevel.height) + copyBoxOffsets.height - copyOrigin.y;

  // Depth/stencil copies must copy whole subresources.
  const isSuccess =
  copyOrigin.x === 0 &&
  copyOrigin.y === 0 &&
  copyWidth === srcSizeAtLevel.width &&
  copyHeight === srcSizeAtLevel.height &&
  copyWidth === dstSizeAtLevel.width &&
  copyHeight === dstSizeAtLevel.height;
  t.TestCopyTextureToTexture(
    { texture: srcTexture, origin: { x: 0, y: 0, z: 0 }, mipLevel: srcCopyLevel },
    { texture: dstTexture, origin: copyOrigin, mipLevel: dstCopyLevel },
    { width: copyWidth, height: copyHeight, depthOrArrayLayers: 1 },
    isSuccess ? 'Success' : 'FinishError'
  );
  t.TestCopyTextureToTexture(
    { texture: srcTexture, origin: copyOrigin, mipLevel: srcCopyLevel },
    { texture: dstTexture, origin: { x: 0, y: 0, z: 0 }, mipLevel: dstCopyLevel },
    { width: copyWidth, height: copyHeight, depthOrArrayLayers: 1 },
    isSuccess ? 'Success' : 'FinishError'
  );
});

g.test('copy_ranges').
desc(
  `
Test that copyTextureToTexture copy boxes must be in range of the subresource.
- for various dimensions
- for various offsets to a full copy for the copy origin/size
- for various copy mip levels
`
).
params((u) =>
u.
combine('dimension', kTextureDimensions)
//.beginSubcases()
.combine('copyBoxOffsets', [
{ x: 0, y: 0, z: 0, width: 0, height: 0, depthOrArrayLayers: -2 },
{ x: 1, y: 0, z: 0, width: 0, height: 0, depthOrArrayLayers: -2 },
{ x: 1, y: 0, z: 0, width: -1, height: 0, depthOrArrayLayers: -2 },
{ x: 0, y: 1, z: 0, width: 0, height: 0, depthOrArrayLayers: -2 },
{ x: 0, y: 1, z: 0, width: 0, height: -1, depthOrArrayLayers: -2 },
{ x: 0, y: 0, z: 1, width: 0, height: 1, depthOrArrayLayers: -2 },
{ x: 0, y: 0, z: 2, width: 0, height: 1, depthOrArrayLayers: 0 },
{ x: 0, y: 0, z: 0, width: 1, height: 0, depthOrArrayLayers: -2 },
{ x: 0, y: 0, z: 0, width: 0, height: 1, depthOrArrayLayers: -2 },
{ x: 0, y: 0, z: 0, width: 0, height: 0, depthOrArrayLayers: 1 },
{ x: 0, y: 0, z: 0, width: 0, height: 0, depthOrArrayLayers: 0 },
{ x: 0, y: 0, z: 1, width: 0, height: 0, depthOrArrayLayers: -1 },
{ x: 0, y: 0, z: 2, width: 0, height: 0, depthOrArrayLayers: -1 }]
).
unless(
  (p) =>
  p.dimension === '1d' && (
  p.copyBoxOffsets.y !== 0 ||
  p.copyBoxOffsets.z !== 0 ||
  p.copyBoxOffsets.height !== 0 ||
  p.copyBoxOffsets.depthOrArrayLayers !== 0)
).
combine('srcCopyLevel', [0, 1, 3]).
combine('dstCopyLevel', [0, 1, 3]).
unless((p) => p.dimension === '1d' && (p.srcCopyLevel !== 0 || p.dstCopyLevel !== 0))
).
fn((t) => {
  const { dimension, copyBoxOffsets, srcCopyLevel, dstCopyLevel } = t.params;

  const textureSize = { width: 16, height: 8, depthOrArrayLayers: 3 };
  let mipLevelCount = 4;
  if (dimension === '1d') {
    mipLevelCount = 1;
    textureSize.height = 1;
    textureSize.depthOrArrayLayers = 1;
  }
  const kFormat = 'rgba8unorm';

  const srcTexture = t.device.createTexture({
    size: textureSize,
    format: kFormat,
    dimension,
    mipLevelCount,
    usage: GPUTextureUsage.COPY_SRC
  });
  const dstTexture = t.device.createTexture({
    size: textureSize,
    format: kFormat,
    dimension,
    mipLevelCount,
    usage: GPUTextureUsage.COPY_DST
  });

  const srcSizeAtLevel = t.GetPhysicalSubresourceSize(
    dimension,
    textureSize,
    kFormat,
    srcCopyLevel
  );
  const dstSizeAtLevel = t.GetPhysicalSubresourceSize(
    dimension,
    textureSize,
    kFormat,
    dstCopyLevel
  );

  const copyOrigin = { x: copyBoxOffsets.x, y: copyBoxOffsets.y, z: copyBoxOffsets.z };

  const copyWidth = Math.max(
    Math.min(srcSizeAtLevel.width, dstSizeAtLevel.width) + copyBoxOffsets.width - copyOrigin.x,
    0
  );
  const copyHeight = Math.max(
    Math.min(srcSizeAtLevel.height, dstSizeAtLevel.height) + copyBoxOffsets.height - copyOrigin.y,
    0
  );
  const copyDepth =
  textureSize.depthOrArrayLayers + copyBoxOffsets.depthOrArrayLayers - copyOrigin.z;

  {
    let isSuccess =
    copyWidth <= srcSizeAtLevel.width &&
    copyHeight <= srcSizeAtLevel.height &&
    copyOrigin.x + copyWidth <= dstSizeAtLevel.width &&
    copyOrigin.y + copyHeight <= dstSizeAtLevel.height;

    if (dimension === '3d') {
      isSuccess =
      isSuccess &&
      copyDepth <= srcSizeAtLevel.depthOrArrayLayers &&
      copyOrigin.z + copyDepth <= dstSizeAtLevel.depthOrArrayLayers;
    } else {
      isSuccess =
      isSuccess &&
      copyDepth <= textureSize.depthOrArrayLayers &&
      copyOrigin.z + copyDepth <= textureSize.depthOrArrayLayers;
    }

    t.TestCopyTextureToTexture(
      { texture: srcTexture, origin: { x: 0, y: 0, z: 0 }, mipLevel: srcCopyLevel },
      { texture: dstTexture, origin: copyOrigin, mipLevel: dstCopyLevel },
      { width: copyWidth, height: copyHeight, depthOrArrayLayers: copyDepth },
      isSuccess ? 'Success' : 'FinishError'
    );
  }

  {
    let isSuccess =
    copyOrigin.x + copyWidth <= srcSizeAtLevel.width &&
    copyOrigin.y + copyHeight <= srcSizeAtLevel.height &&
    copyWidth <= dstSizeAtLevel.width &&
    copyHeight <= dstSizeAtLevel.height;

    if (dimension === '3d') {
      isSuccess =
      isSuccess &&
      copyDepth <= dstSizeAtLevel.depthOrArrayLayers &&
      copyOrigin.z + copyDepth <= srcSizeAtLevel.depthOrArrayLayers;
    } else {
      isSuccess =
      isSuccess &&
      copyDepth <= textureSize.depthOrArrayLayers &&
      copyOrigin.z + copyDepth <= textureSize.depthOrArrayLayers;
    }

    t.TestCopyTextureToTexture(
      { texture: srcTexture, origin: copyOrigin, mipLevel: srcCopyLevel },
      { texture: dstTexture, origin: { x: 0, y: 0, z: 0 }, mipLevel: dstCopyLevel },
      { width: copyWidth, height: copyHeight, depthOrArrayLayers: copyDepth },
      isSuccess ? 'Success' : 'FinishError'
    );
  }
});

g.test('copy_within_same_texture').
desc(
  `
Test that it is an error to use copyTextureToTexture from one subresource to itself.
- for various starting source/destination array layers.
- for various copy sizes in number of array layers

TODO: Extend to check the copy is allowed between different mip levels.
TODO: Extend to 1D and 3D textures.`
).
paramsSubcasesOnly((u) =>
u //
.combine('srcCopyOriginZ', [0, 2, 4]).
combine('dstCopyOriginZ', [0, 2, 4]).
combine('copyExtentDepth', [1, 2, 3])
).
fn((t) => {
  const { srcCopyOriginZ, dstCopyOriginZ, copyExtentDepth } = t.params;

  const kArrayLayerCount = 7;

  const testTexture = t.device.createTexture({
    size: { width: 16, height: 16, depthOrArrayLayers: kArrayLayerCount },
    format: 'rgba8unorm',
    usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.COPY_DST
  });

  const isSuccess =
  Math.min(srcCopyOriginZ, dstCopyOriginZ) + copyExtentDepth <=
  Math.max(srcCopyOriginZ, dstCopyOriginZ);
  t.TestCopyTextureToTexture(
    { texture: testTexture, origin: { x: 0, y: 0, z: srcCopyOriginZ } },
    { texture: testTexture, origin: { x: 0, y: 0, z: dstCopyOriginZ } },
    { width: 16, height: 16, depthOrArrayLayers: copyExtentDepth },
    isSuccess ? 'Success' : 'FinishError'
  );
});

g.test('copy_aspects').
desc(
  `
Test the validations on the member 'aspect' of GPUImageCopyTexture in CopyTextureToTexture().
- for all the color and depth-stencil formats: the texture copy aspects must be both 'all'.
- for all the depth-only formats: the texture copy aspects must be either 'all' or 'depth-only'.
- for all the stencil-only formats: the texture copy aspects must be either 'all' or 'stencil-only'.
`
).
params((u) =>
u.
combine('format', ['rgba8unorm', ...kDepthStencilFormats]).
beginSubcases().
combine('sourceAspect', ['all', 'depth-only', 'stencil-only']).
combine('destinationAspect', ['all', 'depth-only', 'stencil-only'])
).
beforeAllSubcases((t) => {
  const { format } = t.params;
  t.selectDeviceOrSkipTestCase(kTextureFormatInfo[format].feature);
}).
fn((t) => {
  const { format, sourceAspect, destinationAspect } = t.params;

  const kTextureSize = { width: 16, height: 8, depthOrArrayLayers: 1 };

  const srcTexture = t.device.createTexture({
    size: kTextureSize,
    format,
    usage: GPUTextureUsage.COPY_SRC
  });
  const dstTexture = t.device.createTexture({
    size: kTextureSize,
    format,
    usage: GPUTextureUsage.COPY_DST
  });

  // MAINTENANCE_TODO: get the valid aspects from capability_info.ts.
  const kValidAspectsForFormat = {
    rgba8unorm: ['all'],

    // kUnsizedDepthStencilFormats
    depth24plus: ['all', 'depth-only'],
    'depth24plus-stencil8': ['all'],
    'depth32float-stencil8': ['all'],

    // kSizedDepthStencilFormats
    depth32float: ['all', 'depth-only'],
    stencil8: ['all', 'stencil-only'],
    depth16unorm: ['all', 'depth-only']
  };

  const isSourceAspectValid = kValidAspectsForFormat[format].includes(sourceAspect);
  const isDestinationAspectValid = kValidAspectsForFormat[format].includes(destinationAspect);

  t.TestCopyTextureToTexture(
    { texture: srcTexture, origin: { x: 0, y: 0, z: 0 }, aspect: sourceAspect },
    { texture: dstTexture, origin: { x: 0, y: 0, z: 0 }, aspect: destinationAspect },
    kTextureSize,
    isSourceAspectValid && isDestinationAspectValid ? 'Success' : 'FinishError'
  );
});

g.test('copy_ranges_with_compressed_texture_formats').
desc(
  `
Test that copyTextureToTexture copy boxes must be in range of the subresource and aligned to the block size
- for various dimensions
- for various offsets to a full copy for the copy origin/size
- for various copy mip levels

TODO: Express the offsets in "block size" so as to be able to test non-4x4 compressed formats
`
).
params((u) =>
u.
combine('format', kCompressedTextureFormats).
combine('dimension', kTextureDimensions).
filter(({ dimension, format }) => textureDimensionAndFormatCompatible(dimension, format)).
beginSubcases().
combine('copyBoxOffsets', [
{ x: 0, y: 0, z: 0, width: 0, height: 0, depthOrArrayLayers: -2 },
{ x: 1, y: 0, z: 0, width: 0, height: 0, depthOrArrayLayers: -2 },
{ x: 4, y: 0, z: 0, width: 0, height: 0, depthOrArrayLayers: -2 },
{ x: 0, y: 0, z: 0, width: -1, height: 0, depthOrArrayLayers: -2 },
{ x: 0, y: 0, z: 0, width: -4, height: 0, depthOrArrayLayers: -2 },
{ x: 0, y: 1, z: 0, width: 0, height: 0, depthOrArrayLayers: -2 },
{ x: 0, y: 4, z: 0, width: 0, height: 0, depthOrArrayLayers: -2 },
{ x: 0, y: 0, z: 0, width: 0, height: -1, depthOrArrayLayers: -2 },
{ x: 0, y: 0, z: 0, width: 0, height: -4, depthOrArrayLayers: -2 },
{ x: 0, y: 0, z: 0, width: 0, height: 0, depthOrArrayLayers: 0 },
{ x: 0, y: 0, z: 1, width: 0, height: 0, depthOrArrayLayers: -1 }]
).
combine('srcCopyLevel', [0, 1, 2]).
combine('dstCopyLevel', [0, 1, 2])
).
beforeAllSubcases((t) => {
  const { format } = t.params;
  t.selectDeviceOrSkipTestCase(kTextureFormatInfo[format].feature);
  t.skipIfCopyTextureToTextureNotSupportedForFormat(format);
}).
fn((t) => {
  const { format, dimension, copyBoxOffsets, srcCopyLevel, dstCopyLevel } = t.params;
  const { blockWidth, blockHeight } = kTextureFormatInfo[format];

  const kTextureSize = {
    width: 15 * blockWidth,
    height: 12 * blockHeight,
    depthOrArrayLayers: 3
  };
  const kMipLevelCount = 4;

  const srcTexture = t.device.createTexture({
    size: kTextureSize,
    format,
    dimension,
    mipLevelCount: kMipLevelCount,
    usage: GPUTextureUsage.COPY_SRC
  });
  const dstTexture = t.device.createTexture({
    size: kTextureSize,
    format,
    dimension,
    mipLevelCount: kMipLevelCount,
    usage: GPUTextureUsage.COPY_DST
  });

  const srcSizeAtLevel = t.GetPhysicalSubresourceSize(
    dimension,
    kTextureSize,
    format,
    srcCopyLevel
  );
  const dstSizeAtLevel = t.GetPhysicalSubresourceSize(
    dimension,
    kTextureSize,
    format,
    dstCopyLevel
  );

  const copyOrigin = { x: copyBoxOffsets.x, y: copyBoxOffsets.y, z: copyBoxOffsets.z };

  const copyWidth = Math.max(
    Math.min(srcSizeAtLevel.width, dstSizeAtLevel.width) + copyBoxOffsets.width - copyOrigin.x,
    0
  );
  const copyHeight = Math.max(
    Math.min(srcSizeAtLevel.height, dstSizeAtLevel.height) + copyBoxOffsets.height - copyOrigin.y,
    0
  );
  const copyDepth =
  kTextureSize.depthOrArrayLayers + copyBoxOffsets.depthOrArrayLayers - copyOrigin.z;

  const texelBlockWidth = kTextureFormatInfo[format].blockWidth;
  const texelBlockHeight = kTextureFormatInfo[format].blockHeight;

  const isSuccessForCompressedFormats =
  copyOrigin.x % texelBlockWidth === 0 &&
  copyOrigin.y % texelBlockHeight === 0 &&
  copyWidth % texelBlockWidth === 0 &&
  copyHeight % texelBlockHeight === 0;

  {
    const isSuccess =
    isSuccessForCompressedFormats &&
    copyWidth <= srcSizeAtLevel.width &&
    copyHeight <= srcSizeAtLevel.height &&
    copyOrigin.x + copyWidth <= dstSizeAtLevel.width &&
    copyOrigin.y + copyHeight <= dstSizeAtLevel.height &&
    copyOrigin.z + copyDepth <= kTextureSize.depthOrArrayLayers;

    t.TestCopyTextureToTexture(
      { texture: srcTexture, origin: { x: 0, y: 0, z: 0 }, mipLevel: srcCopyLevel },
      { texture: dstTexture, origin: copyOrigin, mipLevel: dstCopyLevel },
      { width: copyWidth, height: copyHeight, depthOrArrayLayers: copyDepth },
      isSuccess ? 'Success' : 'FinishError'
    );
  }

  {
    const isSuccess =
    isSuccessForCompressedFormats &&
    copyOrigin.x + copyWidth <= srcSizeAtLevel.width &&
    copyOrigin.y + copyHeight <= srcSizeAtLevel.height &&
    copyWidth <= dstSizeAtLevel.width &&
    copyHeight <= dstSizeAtLevel.height &&
    copyOrigin.z + copyDepth <= kTextureSize.depthOrArrayLayers;

    t.TestCopyTextureToTexture(
      { texture: srcTexture, origin: copyOrigin, mipLevel: srcCopyLevel },
      { texture: dstTexture, origin: { x: 0, y: 0, z: 0 }, mipLevel: dstCopyLevel },
      { width: copyWidth, height: copyHeight, depthOrArrayLayers: copyDepth },
      isSuccess ? 'Success' : 'FinishError'
    );
  }
});