/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Samples a texture.

- TODO: Test un-encodable formats.
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import {
  isDepthTextureFormat,
  isTextureFormatPossiblyFilterableAsTextureF32,
  kAllTextureFormats,
  kDepthStencilFormats } from
'../../../../../format_info.js';
import { AllFeaturesMaxLimitsGPUTest } from '../../../../../gpu_test.js';

import {
  appendComponentTypeForFormatToTextureType,
  checkCallResults,
  chooseTextureSize,
  createTextureWithRandomDataAndGetTexels,
  doTextureCalls,
  generateSamplePointsCube,
  generateTextureBuiltinInputs1D,
  generateTextureBuiltinInputs2D,
  generateTextureBuiltinInputs3D,
  getDepthOrArrayLayersForViewDimension,
  getTextureTypeForTextureViewDimension,
  isPotentiallyFilterableAndFillable,
  kCubeSamplePointMethods,
  kSamplePointMethods,
  kShortAddressModes,
  kShortAddressModeToAddressMode,
  kShortShaderStages,

  skipIfTextureFormatNotSupportedOrNeedsFilteringAndIsUnfilterable } from




'./texture_utils.js';

export const g = makeTestGroup(AllFeaturesMaxLimitsGPUTest);

g.test('sampled_1d_coords').
specURL('https://www.w3.org/TR/WGSL/#texturesamplelevel').
desc(
  `
fn textureSampleLevel(t: texture_1d<f32>, s: sampler, coords: f32, level: f32) -> vec4<f32>

Parameters:
 * t  The sampled or depth texture to sample.
 * s  The sampler type.
 * coords The texture coordinates used for sampling.
 * level
    * The mip level, with level 0 containing a full size version of the texture.
    * For the functions where level is a f32, fractional values may interpolate between
      two levels if the format is filterable according to the Texture Format Capabilities.
    * When not specified, mip level 0 is sampled.
`
).
params((u) =>
u.
combine('stage', kShortShaderStages).
combine('format', kAllTextureFormats).
filter((t) => isPotentiallyFilterableAndFillable(t.format)).
combine('filt', ['nearest', 'linear']).
filter((t) => t.filt === 'nearest' || isTextureFormatPossiblyFilterableAsTextureF32(t.format)).
combine('modeU', kShortAddressModes).
combine('modeV', kShortAddressModes).
beginSubcases().
combine('samplePoints', kSamplePointMethods)
).
fn(async (t) => {
  const { format, stage, samplePoints, modeU, modeV, filt: minFilter } = t.params;
  t.skipIfTextureFormatAndDimensionNotCompatible(format, '1d');
  skipIfTextureFormatNotSupportedOrNeedsFilteringAndIsUnfilterable(t, minFilter, format);

  // We want at least 4 blocks or something wide enough for 3 mip levels.
  const [width, height] = chooseTextureSize({
    minSize: 8,
    minBlocks: 4,
    format,
    viewDimension: '1d'
  });

  const descriptor = {
    format,
    dimension: '1d',
    size: { width, height },
    usage: GPUTextureUsage.COPY_DST | GPUTextureUsage.TEXTURE_BINDING
  };
  const viewDescriptor = {};
  const { texels, texture } = await createTextureWithRandomDataAndGetTexels(t, descriptor);
  const softwareTexture = { texels, descriptor, viewDescriptor };
  const sampler = {
    addressModeU: kShortAddressModeToAddressMode[modeU],
    addressModeV: kShortAddressModeToAddressMode[modeV],
    minFilter,
    magFilter: minFilter,
    mipmapFilter: minFilter
  };

  const calls = generateTextureBuiltinInputs1D(50, {
    method: samplePoints,
    sampler,
    softwareTexture,
    mipLevel: { num: texture.mipLevelCount, type: 'f32' },
    hashInputs: [stage, format, samplePoints, modeU, modeV, minFilter]
  }).map(({ coords, mipLevel }) => {
    return {
      builtin: 'textureSampleLevel',
      coordType: 'f',
      coords,
      mipLevel,
      levelType: 'f'
    };
  });
  const textureType = appendComponentTypeForFormatToTextureType('texture_1d', format);
  const results = await doTextureCalls(
    t,
    texture,
    viewDescriptor,
    textureType,
    sampler,
    calls,
    stage
  );
  const res = await checkCallResults(
    t,
    { texels, descriptor, viewDescriptor },
    textureType,
    sampler,
    calls,
    results,
    stage,
    texture
  );
  t.expectOK(res);
});

g.test('sampled_2d_coords').
specURL('https://www.w3.org/TR/WGSL/#texturesamplelevel').
desc(
  `
fn textureSampleLevel(t: texture_2d<f32>, s: sampler, coords: vec2<f32>, level: f32) -> vec4<f32>
fn textureSampleLevel(t: texture_2d<f32>, s: sampler, coords: vec2<f32>, level: f32, offset: vec2<i32>) -> vec4<f32>

Parameters:
 * t  The sampled or depth texture to sample.
 * s  The sampler type.
 * coords The texture coordinates used for sampling.
 * level
    * The mip level, with level 0 containing a full size version of the texture.
    * For the functions where level is a f32, fractional values may interpolate between
      two levels if the format is filterable according to the Texture Format Capabilities.
    * When not specified, mip level 0 is sampled.
 * offset
    * The optional texel offset applied to the unnormalized texture coordinate before sampling the texture.
    * This offset is applied before applying any texture wrapping modes.
    * The offset expression must be a creation-time expression (e.g. vec2<i32>(1, 2)).
    * Each offset component must be at least -8 and at most 7.
      Values outside of this range will result in a shader-creation error.
`
).
params((u) =>
u.
combine('stage', kShortShaderStages).
combine('format', kAllTextureFormats).
filter((t) => isPotentiallyFilterableAndFillable(t.format)).
combine('filt', ['nearest', 'linear']).
filter((t) => t.filt === 'nearest' || isTextureFormatPossiblyFilterableAsTextureF32(t.format)).
combine('modeU', kShortAddressModes).
combine('modeV', kShortAddressModes).
combine('offset', [false, true]).
beginSubcases().
combine('samplePoints', kSamplePointMethods)
).
fn(async (t) => {
  const { format, stage, samplePoints, modeU, modeV, filt: minFilter, offset } = t.params;
  skipIfTextureFormatNotSupportedOrNeedsFilteringAndIsUnfilterable(t, minFilter, format);

  // We want at least 4 blocks or something wide enough for 3 mip levels.
  const [width, height] = chooseTextureSize({ minSize: 8, minBlocks: 4, format });
  const descriptor = {
    format,
    size: { width, height },
    mipLevelCount: 3,
    usage: GPUTextureUsage.COPY_DST | GPUTextureUsage.TEXTURE_BINDING
  };
  const viewDescriptor = {};
  const { texels, texture } = await createTextureWithRandomDataAndGetTexels(t, descriptor);
  const softwareTexture = { texels, descriptor, viewDescriptor };
  const sampler = {
    addressModeU: kShortAddressModeToAddressMode[modeU],
    addressModeV: kShortAddressModeToAddressMode[modeV],
    minFilter,
    magFilter: minFilter,
    mipmapFilter: minFilter
  };

  const calls = generateTextureBuiltinInputs2D(50, {
    method: samplePoints,
    sampler,
    softwareTexture,
    mipLevel: { num: texture.mipLevelCount, type: 'f32' },
    offset,
    hashInputs: [stage, format, samplePoints, modeU, modeV, minFilter, offset]
  }).map(({ coords, mipLevel, offset }) => {
    return {
      builtin: 'textureSampleLevel',
      coordType: 'f',
      coords,
      mipLevel,
      levelType: 'f',
      offset
    };
  });
  const textureType = appendComponentTypeForFormatToTextureType('texture_2d', format);
  const results = await doTextureCalls(
    t,
    texture,
    viewDescriptor,
    textureType,
    sampler,
    calls,
    stage
  );
  const res = await checkCallResults(
    t,
    { texels, descriptor, viewDescriptor },
    textureType,
    sampler,
    calls,
    results,
    stage,
    texture
  );
  t.expectOK(res);
});

g.test('sampled_2d_coords,lodClamp').
specURL('https://www.w3.org/TR/WGSL/#texturesamplelevel').
desc(
  `
tests textureSampleLevel with 2d coordinates and various combinations of
baseMipLevel, lodMinClamp, and lodMaxClamp, with an dwithout filtering.
`
).
params((u) =>
u.
combine('stage', kShortShaderStages).
combine('format', kAllTextureFormats).
filter((t) => isPotentiallyFilterableAndFillable(t.format)).
combine('filt', ['nearest', 'linear']).
filter((t) => t.filt === 'nearest' || isTextureFormatPossiblyFilterableAsTextureF32(t.format)).
beginSubcases().
combine('samplePoints', kSamplePointMethods).
combineWithParams([
{ baseMipLevel: 0, lodMinClamp: 0, lodMaxClamp: 2 },
{ baseMipLevel: 0, lodMinClamp: 0.25, lodMaxClamp: 1.75 },
{ baseMipLevel: 1, lodMinClamp: 0, lodMaxClamp: 1 },
{ baseMipLevel: 0, lodMinClamp: 0, lodMaxClamp: 1 },
{ baseMipLevel: 0, lodMinClamp: 1, lodMaxClamp: 2 }]
)
).
fn(async (t) => {
  const {
    format,
    stage,
    samplePoints,
    filt: minFilter,
    baseMipLevel,
    lodMaxClamp,
    lodMinClamp
  } = t.params;
  skipIfTextureFormatNotSupportedOrNeedsFilteringAndIsUnfilterable(t, minFilter, format);

  // We want at least 4 blocks or something wide enough for 3 mip levels.
  const [width, height] = chooseTextureSize({ minSize: 8, minBlocks: 4, format });
  const descriptor = {
    format,
    size: { width, height },
    mipLevelCount: 3,
    usage: GPUTextureUsage.COPY_DST | GPUTextureUsage.TEXTURE_BINDING
  };
  const viewDescriptor = { baseMipLevel };
  const { texels, texture } = await createTextureWithRandomDataAndGetTexels(t, descriptor);
  const softwareTexture = { texels, descriptor, viewDescriptor };
  const sampler = {
    minFilter,
    magFilter: minFilter,
    mipmapFilter: minFilter,
    lodMinClamp,
    lodMaxClamp
  };

  const calls = generateTextureBuiltinInputs2D(50, {
    method: samplePoints,
    sampler,
    softwareTexture,
    mipLevel: { num: texture.mipLevelCount, type: 'f32' },
    hashInputs: [stage, format, samplePoints, minFilter, baseMipLevel, lodMinClamp, lodMaxClamp]
  }).map(({ coords, mipLevel, offset }) => {
    return {
      builtin: 'textureSampleLevel',
      coordType: 'f',
      coords,
      mipLevel,
      levelType: 'f',
      offset
    };
  });
  const textureType = appendComponentTypeForFormatToTextureType('texture_2d', format);
  const results = await doTextureCalls(
    t,
    texture,
    viewDescriptor,
    textureType,
    sampler,
    calls,
    stage
  );
  const res = await checkCallResults(
    t,
    { texels, descriptor, viewDescriptor },
    textureType,
    sampler,
    calls,
    results,
    stage,
    texture
  );
  t.expectOK(res);
});

g.test('sampled_array_2d_coords').
specURL('https://www.w3.org/TR/WGSL/#texturesamplelevel').
desc(
  `
C is i32 or u32

fn textureSampleLevel(t: texture_2d_array<f32>, s: sampler, coords: vec2<f32>, array_index: A, level: f32) -> vec4<f32>
fn textureSampleLevel(t: texture_2d_array<f32>, s: sampler, coords: vec2<f32>, array_index: A, level: f32, offset: vec2<i32>) -> vec4<f32>

Parameters:
 * t  The sampled or depth texture to sample.
 * s  The sampler type.
 * coords The texture coordinates used for sampling.
 * array_index The 0-based texture array index to sample.
 * level
    * The mip level, with level 0 containing a full size version of the texture.
    * For the functions where level is a f32, fractional values may interpolate between
      two levels if the format is filterable according to the Texture Format Capabilities.
    * When not specified, mip level 0 is sampled.
 * offset
    * The optional texel offset applied to the unnormalized texture coordinate before sampling the texture.
    * This offset is applied before applying any texture wrapping modes.
    * The offset expression must be a creation-time expression (e.g. vec2<i32>(1, 2)).
    * Each offset component must be at least -8 and at most 7.
      Values outside of this range will result in a shader-creation error.
`
).
params((u) =>
u.
combine('stage', kShortShaderStages).
combine('format', kAllTextureFormats).
filter((t) => isPotentiallyFilterableAndFillable(t.format)).
combine('filt', ['nearest', 'linear']).
filter((t) => t.filt === 'nearest' || isTextureFormatPossiblyFilterableAsTextureF32(t.format)).
combine('modeU', kShortAddressModes).
combine('modeV', kShortAddressModes).
combine('offset', [false, true]).
beginSubcases().
combine('samplePoints', kSamplePointMethods).
combine('A', ['i32', 'u32']).
combine('depthOrArrayLayers', [1, 8])
).
fn(async (t) => {
  const {
    format,
    stage,
    samplePoints,
    A,
    modeU,
    modeV,
    filt: minFilter,
    offset,
    depthOrArrayLayers
  } = t.params;
  skipIfTextureFormatNotSupportedOrNeedsFilteringAndIsUnfilterable(t, minFilter, format);

  // We want at least 4 blocks or something wide enough for 3 mip levels.
  const [width, height] = chooseTextureSize({ minSize: 8, minBlocks: 4, format });

  const descriptor = {
    format,
    size: { width, height, depthOrArrayLayers },
    mipLevelCount: 3,
    usage: GPUTextureUsage.COPY_DST | GPUTextureUsage.TEXTURE_BINDING,
    ...(t.isCompatibility && { textureBindingViewDimension: '2d-array' })
  };
  const { texels, texture } = await createTextureWithRandomDataAndGetTexels(t, descriptor);
  const sampler = {
    addressModeU: kShortAddressModeToAddressMode[modeU],
    addressModeV: kShortAddressModeToAddressMode[modeV],
    minFilter,
    magFilter: minFilter,
    mipmapFilter: minFilter
  };

  const calls = generateTextureBuiltinInputs2D(50, {
    method: samplePoints,
    sampler,
    descriptor,
    mipLevel: { num: texture.mipLevelCount, type: 'f32' },
    arrayIndex: { num: texture.depthOrArrayLayers, type: A },
    offset,
    hashInputs: [stage, format, samplePoints, A, modeU, modeV, minFilter, offset]
  }).map(({ coords, mipLevel, arrayIndex, offset }) => {
    return {
      builtin: 'textureSampleLevel',
      coordType: 'f',
      coords,
      mipLevel,
      levelType: 'f',
      arrayIndex,
      arrayIndexType: A === 'i32' ? 'i' : 'u',
      offset
    };
  });
  const textureType = appendComponentTypeForFormatToTextureType('texture_2d_array', format);
  const viewDescriptor = { dimension: '2d-array' };
  const results = await doTextureCalls(
    t,
    texture,
    viewDescriptor,
    textureType,
    sampler,
    calls,
    stage
  );
  const res = await checkCallResults(
    t,
    { texels, descriptor, viewDescriptor },
    textureType,
    sampler,
    calls,
    results,
    stage,
    texture
  );
  t.expectOK(res);
});

g.test('sampled_3d_coords').
specURL('https://www.w3.org/TR/WGSL/#texturesamplelevel').
desc(
  `
fn textureSampleLevel(t: texture_3d<f32>, s: sampler, coords: vec3<f32>, level: f32) -> vec4<f32>
fn textureSampleLevel(t: texture_3d<f32>, s: sampler, coords: vec3<f32>, level: f32, offset: vec3<i32>) -> vec4<f32>
fn textureSampleLevel(t: texture_cube<f32>, s: sampler, coords: vec3<f32>, level: f32) -> vec4<f32>

Parameters:
 * t  The sampled or depth texture to sample.
 * s  The sampler type.
 * coords The texture coordinates used for sampling.
 * level
    * The mip level, with level 0 containing a full size version of the texture.
    * For the functions where level is a f32, fractional values may interpolate between
      two levels if the format is filterable according to the Texture Format Capabilities.
    * When not specified, mip level 0 is sampled.
 * offset
    * The optional texel offset applied to the unnormalized texture coordinate before sampling the texture.
    * This offset is applied before applying any texture wrapping modes.
    * The offset expression must be a creation-time expression (e.g. vec2<i32>(1, 2)).
    * Each offset component must be at least -8 and at most 7.
      Values outside of this range will result in a shader-creation error.
`
).
params((u) =>
u.
combine('stage', kShortShaderStages).
combine('format', kAllTextureFormats).
filter((t) => isPotentiallyFilterableAndFillable(t.format)).
combine('dim', ['3d', 'cube']).
combine('filt', ['nearest', 'linear']).
filter((t) => t.filt === 'nearest' || isTextureFormatPossiblyFilterableAsTextureF32(t.format)).
combine('mode', kShortAddressModes).
combine('offset', [false, true]).
filter((t) => t.dim !== 'cube' || t.offset !== true).
beginSubcases().
combine('samplePoints', kCubeSamplePointMethods).
filter((t) => t.samplePoints !== 'cube-edges' || t.dim !== '3d')
).
fn(async (t) => {
  const {
    format,
    dim: viewDimension,
    stage,
    samplePoints,
    mode,
    filt: minFilter,
    offset
  } = t.params;
  skipIfTextureFormatNotSupportedOrNeedsFilteringAndIsUnfilterable(t, minFilter, format);
  t.skipIfTextureFormatAndViewDimensionNotCompatible(format, viewDimension);

  const [width, height] = chooseTextureSize({ minSize: 32, minBlocks: 2, format, viewDimension });
  const depthOrArrayLayers = getDepthOrArrayLayersForViewDimension(viewDimension);

  const descriptor = {
    format,
    dimension: viewDimension === '3d' ? '3d' : '2d',
    ...(t.isCompatibility && { textureBindingViewDimension: viewDimension }),
    size: { width, height, depthOrArrayLayers },
    usage: GPUTextureUsage.COPY_DST | GPUTextureUsage.TEXTURE_BINDING,
    mipLevelCount: 3
  };
  const viewDescriptor = {
    dimension: viewDimension
  };
  const { texels, texture } = await createTextureWithRandomDataAndGetTexels(t, descriptor);
  const softwareTexture = { texels, descriptor, viewDescriptor };
  const sampler = {
    addressModeU: kShortAddressModeToAddressMode[mode],
    addressModeV: kShortAddressModeToAddressMode[mode],
    addressModeW: kShortAddressModeToAddressMode[mode],
    minFilter,
    magFilter: minFilter,
    mipmapFilter: minFilter
  };
  const hashInputs = [stage, format, viewDimension, samplePoints, mode, minFilter, offset];
  const calls = (
  viewDimension === '3d' ?
  generateTextureBuiltinInputs3D(50, {
    method: samplePoints,
    sampler,
    softwareTexture,
    mipLevel: { num: texture.mipLevelCount, type: 'f32' },
    offset,
    hashInputs
  }) :
  generateSamplePointsCube(50, {
    method: samplePoints,
    sampler,
    softwareTexture,
    mipLevel: { num: texture.mipLevelCount, type: 'f32' },
    hashInputs
  })).
  map(({ coords, mipLevel, offset }) => {
    return {
      builtin: 'textureSampleLevel',
      coordType: 'f',
      coords,
      mipLevel,
      levelType: 'f',
      offset
    };
  });
  const textureType = getTextureTypeForTextureViewDimension(viewDimension);
  const results = await doTextureCalls(
    t,
    texture,
    viewDescriptor,
    textureType,
    sampler,
    calls,
    stage
  );
  const res = await checkCallResults(
    t,
    { texels, descriptor, viewDescriptor },
    textureType,
    sampler,
    calls,
    results,
    stage,
    texture
  );
  t.expectOK(res);
});

g.test('sampled_3d_coords,lodClamp').
specURL('https://www.w3.org/TR/WGSL/#texturesamplelevel').
desc(
  `
tests textureSampleLevel with 3d coordinates and various combinations of
baseMipLevel, lodMinClamp, and lodMaxClamp, with an dwithout filtering.
`
).
params((u) =>
u.
combine('stage', kShortShaderStages).
combine('format', kAllTextureFormats).
filter((t) => isPotentiallyFilterableAndFillable(t.format)).
combine('dim', ['3d', 'cube']).
combine('filt', ['nearest', 'linear']).
filter((t) => t.filt === 'nearest' || isTextureFormatPossiblyFilterableAsTextureF32(t.format)).
beginSubcases().
combine('samplePoints', kCubeSamplePointMethods).
filter((t) => t.samplePoints !== 'cube-edges' || t.dim !== '3d').
combineWithParams([
{ baseMipLevel: 0, lodMinClamp: 0, lodMaxClamp: 2 },
{ baseMipLevel: 0, lodMinClamp: 0.25, lodMaxClamp: 1.75 },
{ baseMipLevel: 1, lodMinClamp: 0, lodMaxClamp: 1 },
{ baseMipLevel: 0, lodMinClamp: 0, lodMaxClamp: 1 },
{ baseMipLevel: 0, lodMinClamp: 1, lodMaxClamp: 2 }]
)
).
fn(async (t) => {
  const {
    format,
    dim: viewDimension,
    stage,
    samplePoints,
    filt: minFilter,
    baseMipLevel,
    lodMaxClamp,
    lodMinClamp
  } = t.params;
  skipIfTextureFormatNotSupportedOrNeedsFilteringAndIsUnfilterable(t, minFilter, format);
  t.skipIfTextureFormatAndViewDimensionNotCompatible(format, viewDimension);

  const [width, height] = chooseTextureSize({ minSize: 32, minBlocks: 2, format, viewDimension });
  const depthOrArrayLayers = getDepthOrArrayLayersForViewDimension(viewDimension);

  const descriptor = {
    format,
    dimension: viewDimension === '3d' ? '3d' : '2d',
    ...(t.isCompatibility && { textureBindingViewDimension: viewDimension }),
    size: { width, height, depthOrArrayLayers },
    usage: GPUTextureUsage.COPY_DST | GPUTextureUsage.TEXTURE_BINDING,
    mipLevelCount: 3
  };
  const viewDescriptor = {
    dimension: viewDimension,
    baseMipLevel
  };
  const { texels, texture } = await createTextureWithRandomDataAndGetTexels(t, descriptor);
  const softwareTexture = { texels, descriptor, viewDescriptor };
  const sampler = {
    minFilter,
    magFilter: minFilter,
    mipmapFilter: minFilter,
    lodMinClamp,
    lodMaxClamp
  };
  const hashInputs = [
  stage,
  format,
  viewDimension,
  samplePoints,
  minFilter,
  baseMipLevel,
  lodMinClamp,
  lodMaxClamp];

  const calls = (
  viewDimension === '3d' ?
  generateTextureBuiltinInputs3D(50, {
    method: samplePoints,
    sampler,
    softwareTexture,
    mipLevel: { num: texture.mipLevelCount, type: 'f32' },
    hashInputs
  }) :
  generateSamplePointsCube(50, {
    method: samplePoints,
    sampler,
    softwareTexture,
    mipLevel: { num: texture.mipLevelCount, type: 'f32' },
    hashInputs
  })).
  map(({ coords, mipLevel, offset }) => {
    return {
      builtin: 'textureSampleLevel',
      coordType: 'f',
      coords,
      mipLevel,
      levelType: 'f',
      offset
    };
  });
  const textureType = getTextureTypeForTextureViewDimension(viewDimension);
  const results = await doTextureCalls(
    t,
    texture,
    viewDescriptor,
    textureType,
    sampler,
    calls,
    stage
  );
  const res = await checkCallResults(
    t,
    { texels, descriptor, viewDescriptor },
    textureType,
    sampler,
    calls,
    results,
    stage,
    texture
  );
  t.expectOK(res);
});

g.test('sampled_array_3d_coords').
specURL('https://www.w3.org/TR/WGSL/#texturesamplelevel').
desc(
  `
A is i32 or u32

fn textureSampleLevel(t: texture_cube_array<f32>, s: sampler, coords: vec3<f32>, array_index: A, level: f32) -> vec4<f32>

Parameters:
 * t  The sampled or depth texture to sample.
 * s  The sampler type.
 * coords The texture coordinates used for sampling.
 * array_index The 0-based texture array index to sample.
 * level
    * The mip level, with level 0 containing a full size version of the texture.
    * For the functions where level is a f32, fractional values may interpolate between
      two levels if the format is filterable according to the Texture Format Capabilities.
    * When not specified, mip level 0 is sampled.

- TODO: set mipLevelCount to 3 for cubemaps. See MAINTENANCE_TODO below

  The issue is sampling a corner of a cubemap is undefined. We try to quantize coordinates
  so we never get a corner but when sampling smaller mip levels that's more difficult.

  * Solution 1: Fix the quantization
  * Solution 2: special case checking cube corners. Expect some value between the color of the 3 corner texels.
`
).
params((u) =>
u.
combine('stage', kShortShaderStages).
combine('format', kAllTextureFormats).
filter((t) => isPotentiallyFilterableAndFillable(t.format)).
combine('filt', ['nearest', 'linear']).
filter((t) => t.filt === 'nearest' || isTextureFormatPossiblyFilterableAsTextureF32(t.format)).
combine('mode', kShortAddressModes).
beginSubcases().
combine('samplePoints', kCubeSamplePointMethods).
combine('A', ['i32', 'u32'])
).
fn(async (t) => {
  const { format, stage, samplePoints, A, mode, filt: minFilter } = t.params;
  skipIfTextureFormatNotSupportedOrNeedsFilteringAndIsUnfilterable(t, minFilter, format);
  t.skipIfTextureViewDimensionNotSupported('cube-array');

  const viewDimension = 'cube-array';
  const size = chooseTextureSize({
    minSize: 32,
    minBlocks: 4,
    format,
    viewDimension
  });
  const descriptor = {
    format,
    size,
    usage: GPUTextureUsage.COPY_DST | GPUTextureUsage.TEXTURE_BINDING,
    mipLevelCount: 3
  };
  const { texels, texture } = await createTextureWithRandomDataAndGetTexels(t, descriptor);
  const sampler = {
    addressModeU: kShortAddressModeToAddressMode[mode],
    addressModeV: kShortAddressModeToAddressMode[mode],
    addressModeW: kShortAddressModeToAddressMode[mode],
    minFilter,
    magFilter: minFilter,
    mipmapFilter: minFilter
  };

  const calls = generateSamplePointsCube(50, {
    method: samplePoints,
    sampler,
    descriptor,
    mipLevel: { num: texture.mipLevelCount, type: 'f32' },
    arrayIndex: { num: texture.depthOrArrayLayers / 6, type: A },
    hashInputs: [stage, format, viewDimension, A, samplePoints, mode, minFilter]
  }).map(({ coords, mipLevel, arrayIndex }) => {
    return {
      builtin: 'textureSampleLevel',
      coordType: 'f',
      coords,
      mipLevel,
      levelType: 'f',
      arrayIndex,
      arrayIndexType: A === 'i32' ? 'i' : 'u'
    };
  });
  const viewDescriptor = {
    dimension: viewDimension
  };
  const textureType = getTextureTypeForTextureViewDimension(viewDimension);
  const results = await doTextureCalls(
    t,
    texture,
    viewDescriptor,
    textureType,
    sampler,
    calls,
    stage
  );
  const res = await checkCallResults(
    t,
    { texels, descriptor, viewDescriptor },
    textureType,
    sampler,
    calls,
    results,
    stage,
    texture
  );
  t.expectOK(res);
});

g.test('depth_2d_coords').
specURL('https://www.w3.org/TR/WGSL/#texturesamplelevel').
desc(
  `
L is i32 or u32

fn textureSampleLevel(t: texture_depth_2d, s: sampler, coords: vec2<f32>, level: L) -> f32
fn textureSampleLevel(t: texture_depth_2d, s: sampler, coords: vec2<f32>, level: L, offset: vec2<i32>) -> f32

Parameters:
 * t  The sampled or depth texture to sample.
 * s  The sampler type.
 * coords The texture coordinates used for sampling.
 * level
    * The mip level, with level 0 containing a full size version of the texture.
    * For the functions where level is a f32, fractional values may interpolate between
      two levels if the format is filterable according to the Texture Format Capabilities.
    * When not specified, mip level 0 is sampled.
 * offset
    * The optional texel offset applied to the unnormalized texture coordinate before sampling the texture.
    * This offset is applied before applying any texture wrapping modes.
    * The offset expression must be a creation-time expression (e.g. vec2<i32>(1, 2)).
    * Each offset component must be at least -8 and at most 7.
      Values outside of this range will result in a shader-creation error.
`
).
params((u) =>
u.
combine('stage', kShortShaderStages).
combine('format', kDepthStencilFormats)
// filter out stencil only formats
.filter((t) => isDepthTextureFormat(t.format)).
combine('mode', kShortAddressModes).
combine('offset', [false, true]).
beginSubcases().
combine('samplePoints', kSamplePointMethods).
combine('L', ['i32', 'u32'])
).
fn(async (t) => {
  const { format, stage, samplePoints, mode, L, offset } = t.params;
  t.skipIfTextureFormatNotSupported(format);
  t.skipIfDepthTextureCanNotBeUsedWithNonComparisonSampler();

  // We want at least 4 blocks or something wide enough for 3 mip levels.
  const [width, height] = chooseTextureSize({ minSize: 8, minBlocks: 4, format });
  const descriptor = {
    format,
    size: { width, height },
    mipLevelCount: 3,
    usage: GPUTextureUsage.COPY_DST | GPUTextureUsage.TEXTURE_BINDING
  };
  const { texels, texture } = await createTextureWithRandomDataAndGetTexels(t, descriptor);
  const sampler = {
    addressModeU: kShortAddressModeToAddressMode[mode],
    addressModeV: kShortAddressModeToAddressMode[mode]
  };

  const calls = generateTextureBuiltinInputs2D(50, {
    method: samplePoints,
    sampler,
    descriptor,
    mipLevel: { num: texture.mipLevelCount, type: L },
    offset,
    hashInputs: [stage, format, samplePoints, mode, L, offset]
  }).map(({ coords, mipLevel, offset }) => {
    return {
      builtin: 'textureSampleLevel',
      coordType: 'f',
      coords,
      mipLevel,
      levelType: L === 'i32' ? 'i' : 'u',
      offset
    };
  });
  const textureType = appendComponentTypeForFormatToTextureType('texture_depth_2d', format);
  const viewDescriptor = {};
  const results = await doTextureCalls(
    t,
    texture,
    viewDescriptor,
    textureType,
    sampler,
    calls,
    stage
  );
  const res = await checkCallResults(
    t,
    { texels, descriptor, viewDescriptor },
    textureType,
    sampler,
    calls,
    results,
    stage,
    texture
  );
  t.expectOK(res);
});

g.test('depth_array_2d_coords').
specURL('https://www.w3.org/TR/WGSL/#texturesamplelevel').
desc(
  `
A is i32 or u32
L is i32 or u32

fn textureSampleLevel(t: texture_depth_2d_array, s: sampler, coords: vec2<f32>, array_index: A, level: L) -> f32
fn textureSampleLevel(t: texture_depth_2d_array, s: sampler, coords: vec2<f32>, array_index: A, level: L, offset: vec2<i32>) -> f32

Parameters:
 * t  The sampled or depth texture to sample.
 * s  The sampler type.
 * array_index The 0-based texture array index to sample.
 * coords The texture coordinates used for sampling.
 * level
    * The mip level, with level 0 containing a full size version of the texture.
    * For the functions where level is a f32, fractional values may interpolate between
      two levels if the format is filterable according to the Texture Format Capabilities.
    * When not specified, mip level 0 is sampled.
 * offset
    * The optional texel offset applied to the unnormalized texture coordinate before sampling the texture.
    * This offset is applied before applying any texture wrapping modes.
    * The offset expression must be a creation-time expression (e.g. vec2<i32>(1, 2)).
    * Each offset component must be at least -8 and at most 7.
      Values outside of this range will result in a shader-creation error.
`
).
params((u) =>
u.
combine('stage', kShortShaderStages).
combine('format', kDepthStencilFormats)
// filter out stencil only formats
.filter((t) => isDepthTextureFormat(t.format)).
combine('mode', kShortAddressModes).
combine('offset', [false, true]).
beginSubcases().
combine('samplePoints', kSamplePointMethods).
combine('A', ['i32', 'u32']).
combine('L', ['i32', 'u32']).
combine('depthOrArrayLayers', [1, 8])
).
fn(async (t) => {
  const { format, stage, samplePoints, mode, A, L, offset, depthOrArrayLayers } = t.params;
  t.skipIfTextureFormatNotSupported(format);
  t.skipIfDepthTextureCanNotBeUsedWithNonComparisonSampler();

  // We want at least 4 blocks or something wide enough for 3 mip levels.
  const [width, height] = chooseTextureSize({ minSize: 8, minBlocks: 4, format });
  const descriptor = {
    format,
    size: { width, height, depthOrArrayLayers },
    mipLevelCount: 3,
    usage: GPUTextureUsage.COPY_DST | GPUTextureUsage.TEXTURE_BINDING,
    ...(t.isCompatibility && { textureBindingViewDimension: '2d-array' })
  };
  const { texels, texture } = await createTextureWithRandomDataAndGetTexels(t, descriptor);
  const sampler = {
    addressModeU: kShortAddressModeToAddressMode[mode],
    addressModeV: kShortAddressModeToAddressMode[mode]
  };

  const calls = generateTextureBuiltinInputs2D(50, {
    method: samplePoints,
    sampler,
    descriptor,
    arrayIndex: { num: texture.depthOrArrayLayers, type: A },
    mipLevel: { num: texture.mipLevelCount, type: L },
    offset,
    hashInputs: [stage, format, samplePoints, mode, L, A, offset]
  }).map(({ coords, mipLevel, arrayIndex, offset }) => {
    return {
      builtin: 'textureSampleLevel',
      coordType: 'f',
      coords,
      mipLevel,
      levelType: L === 'i32' ? 'i' : 'u',
      arrayIndex,
      arrayIndexType: A === 'i32' ? 'i' : 'u',
      offset
    };
  });
  const textureType = appendComponentTypeForFormatToTextureType('texture_depth_2d_array', format);
  const viewDescriptor = { dimension: '2d-array' };
  const results = await doTextureCalls(
    t,
    texture,
    viewDescriptor,
    textureType,
    sampler,
    calls,
    stage
  );
  const res = await checkCallResults(
    t,
    { texels, descriptor, viewDescriptor },
    textureType,
    sampler,
    calls,
    results,
    stage,
    texture
  );
  t.expectOK(res);
});

g.test('depth_3d_coords').
specURL('https://www.w3.org/TR/WGSL/#texturesamplelevel').
desc(
  `
L is i32 or u32
A is i32 or u32

fn textureSampleLevel(t: texture_depth_cube, s: sampler, coords: vec3<f32>, level: L) -> f32
fn textureSampleLevel(t: texture_depth_cube_array, s: sampler, coords: vec3<f32>, array_index: A, level: L) -> f32

Parameters:
 * t  The sampled or depth texture to sample.
 * s  The sampler type.
 * coords The texture coordinates used for sampling.
 * level
    * The mip level, with level 0 containing a full size version of the texture.
    * For the functions where level is a f32, fractional values may interpolate between
      two levels if the format is filterable according to the Texture Format Capabilities.
    * When not specified, mip level 0 is sampled.
 * offset
    * The optional texel offset applied to the unnormalized texture coordinate before sampling the texture.
    * This offset is applied before applying any texture wrapping modes.
    * The offset expression must be a creation-time expression (e.g. vec2<i32>(1, 2)).
    * Each offset component must be at least -8 and at most 7.
      Values outside of this range will result in a shader-creation error.
`
).
params((u) =>
u.
combine('stage', kShortShaderStages).
combine('format', kDepthStencilFormats)
// filter out stencil only formats
.filter((t) => isDepthTextureFormat(t.format)).
combineWithParams([
{ viewDimension: 'cube' },
{ viewDimension: 'cube-array', A: 'i32' },
{ viewDimension: 'cube-array', A: 'u32' }]
).
combine('mode', kShortAddressModes).
beginSubcases().
combine('samplePoints', kCubeSamplePointMethods).
combine('L', ['i32', 'u32'])
).
fn(async (t) => {
  const { format, stage, viewDimension, samplePoints, A, L, mode } = t.params;
  t.skipIfTextureFormatNotSupported(format);
  t.skipIfDepthTextureCanNotBeUsedWithNonComparisonSampler();
  t.skipIfTextureViewDimensionNotSupported(viewDimension);

  const size = chooseTextureSize({
    minSize: 32,
    minBlocks: 4,
    format,
    viewDimension
  });
  const descriptor = {
    format,
    size,
    usage: GPUTextureUsage.COPY_DST | GPUTextureUsage.TEXTURE_BINDING,
    mipLevelCount: 3,
    ...(t.isCompatibility && { textureBindingViewDimension: viewDimension })
  };
  const { texels, texture } = await createTextureWithRandomDataAndGetTexels(t, descriptor);
  const sampler = {
    addressModeU: kShortAddressModeToAddressMode[mode],
    addressModeV: kShortAddressModeToAddressMode[mode],
    addressModeW: kShortAddressModeToAddressMode[mode]
  };

  const calls = generateSamplePointsCube(50, {
    method: samplePoints,
    sampler,
    descriptor,
    mipLevel: { num: texture.mipLevelCount - 1, type: L },
    arrayIndex: A ? { num: texture.depthOrArrayLayers / 6, type: A } : undefined,
    hashInputs: [stage, format, viewDimension, samplePoints, mode]
  }).map(({ coords, mipLevel, arrayIndex }) => {
    return {
      builtin: 'textureSampleLevel',
      coordType: 'f',
      coords,
      mipLevel,
      levelType: L === 'i32' ? 'i' : 'u',
      arrayIndex,
      arrayIndexType: A ? A === 'i32' ? 'i' : 'u' : undefined
    };
  });
  const viewDescriptor = {
    dimension: viewDimension
  };
  const textureType =
  viewDimension === 'cube' ? 'texture_depth_cube' : 'texture_depth_cube_array';
  const results = await doTextureCalls(
    t,
    texture,
    viewDescriptor,
    textureType,
    sampler,
    calls,
    stage
  );

  const res = await checkCallResults(
    t,
    { texels, descriptor, viewDescriptor },
    textureType,
    sampler,
    calls,
    results,
    stage,
    texture
  );
  t.expectOK(res);
});