/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Samples a texture.

- TODO: Test un-encodable formats.
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import {
  isDepthTextureFormat,
  isEncodableTextureFormat,
  kCompressedTextureFormats,
  kDepthStencilFormats,
  kEncodableTextureFormats } from
'../../../../../format_info.js';

import {
  appendComponentTypeForFormatToTextureType,
  checkCallResults,
  chooseTextureSize,
  createTextureWithRandomDataAndGetTexels,
  doTextureCalls,
  generateSamplePointsCube,
  generateTextureBuiltinInputs2D,
  generateTextureBuiltinInputs3D,
  getDepthOrArrayLayersForViewDimension,
  getTextureTypeForTextureViewDimension,
  isPotentiallyFilterableAndFillable,
  isSupportedViewFormatCombo,
  kCubeSamplePointMethods,
  kSamplePointMethods,
  kShortAddressModes,
  kShortAddressModeToAddressMode,
  kShortShaderStages,

  skipIfNeedsFilteringAndIsUnfilterable,
  skipIfTextureFormatNotSupportedNotAvailableOrNotFilterable,



  WGSLTextureSampleTest } from
'./texture_utils.js';

const kTestableColorFormats = [...kEncodableTextureFormats, ...kCompressedTextureFormats];

export const g = makeTestGroup(WGSLTextureSampleTest);

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
combine('format', kTestableColorFormats).
filter((t) => isPotentiallyFilterableAndFillable(t.format)).
combine('filt', ['nearest', 'linear']).
combine('modeU', kShortAddressModes).
combine('modeV', kShortAddressModes).
combine('offset', [false, true]).
beginSubcases().
combine('samplePoints', kSamplePointMethods)
).
beforeAllSubcases((t) =>
skipIfTextureFormatNotSupportedNotAvailableOrNotFilterable(t, t.params.format)
).
fn(async (t) => {
  const { format, stage, samplePoints, modeU, modeV, filt: minFilter, offset } = t.params;
  skipIfNeedsFilteringAndIsUnfilterable(t, minFilter, format);

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
combine('format', kTestableColorFormats).
filter((t) => isPotentiallyFilterableAndFillable(t.format)).
combine('filt', ['nearest', 'linear']).
combine('modeU', kShortAddressModes).
combine('modeV', kShortAddressModes).
combine('offset', [false, true]).
beginSubcases().
combine('samplePoints', kSamplePointMethods).
combine('A', ['i32', 'u32'])
).
beforeAllSubcases((t) =>
skipIfTextureFormatNotSupportedNotAvailableOrNotFilterable(t, t.params.format)
).
fn(async (t) => {
  const { format, stage, samplePoints, A, modeU, modeV, filt: minFilter, offset } = t.params;
  skipIfNeedsFilteringAndIsUnfilterable(t, minFilter, format);

  // We want at least 4 blocks or something wide enough for 3 mip levels.
  const [width, height] = chooseTextureSize({ minSize: 8, minBlocks: 4, format });
  const depthOrArrayLayers = 4;

  const descriptor = {
    format,
    size: { width, height, depthOrArrayLayers },
    mipLevelCount: 3,
    usage: GPUTextureUsage.COPY_DST | GPUTextureUsage.TEXTURE_BINDING
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
combine('format', kTestableColorFormats).
filter((t) => isPotentiallyFilterableAndFillable(t.format)).
combine('dim', ['3d', 'cube']).
filter((t) => isSupportedViewFormatCombo(t.format, t.dim)).
combine('filt', ['nearest', 'linear']).
combine('mode', kShortAddressModes).
combine('offset', [false, true]).
filter((t) => t.dim !== 'cube' || t.offset !== true).
beginSubcases().
combine('samplePoints', kCubeSamplePointMethods).
filter((t) => t.samplePoints !== 'cube-edges' || t.dim !== '3d')
).
beforeAllSubcases((t) =>
skipIfTextureFormatNotSupportedNotAvailableOrNotFilterable(t, t.params.format)
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
  skipIfNeedsFilteringAndIsUnfilterable(t, minFilter, format);

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
  const { texels, texture } = await createTextureWithRandomDataAndGetTexels(t, descriptor);
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
    descriptor,
    mipLevel: { num: texture.mipLevelCount, type: 'f32' },
    offset,
    hashInputs
  }) :
  generateSamplePointsCube(50, {
    method: samplePoints,
    sampler,
    descriptor,
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
combine('format', kTestableColorFormats).
filter((t) => isPotentiallyFilterableAndFillable(t.format)).
combine('filt', ['nearest', 'linear']).
combine('mode', kShortAddressModes).
beginSubcases().
combine('samplePoints', kCubeSamplePointMethods).
combine('A', ['i32', 'u32'])
).
beforeAllSubcases((t) => {
  skipIfTextureFormatNotSupportedNotAvailableOrNotFilterable(t, t.params.format);
  t.skipIfTextureViewDimensionNotSupported('cube-array');
}).
fn(async (t) => {
  const { format, stage, samplePoints, A, mode, filt: minFilter } = t.params;
  skipIfNeedsFilteringAndIsUnfilterable(t, minFilter, format);

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
.filter((t) => isDepthTextureFormat(t.format))
// MAINTENANCE_TODO: Remove when support for depth24plus, depth24plus-stencil8, and depth32float-stencil8 is added.
.filter((t) => isEncodableTextureFormat(t.format)).
combine('filt', ['nearest', 'linear']).
combine('mode', kShortAddressModes).
combine('offset', [false, true]).
beginSubcases().
combine('samplePoints', kSamplePointMethods).
combine('L', ['i32', 'u32'])
).
beforeAllSubcases((t) =>
skipIfTextureFormatNotSupportedNotAvailableOrNotFilterable(t, t.params.format)
).
fn(async (t) => {
  const { format, stage, samplePoints, mode, filt: minFilter, L, offset } = t.params;

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
    addressModeV: kShortAddressModeToAddressMode[mode],
    minFilter,
    magFilter: minFilter,
    mipmapFilter: minFilter
  };

  const calls = generateTextureBuiltinInputs2D(50, {
    method: samplePoints,
    sampler,
    descriptor,
    mipLevel: { num: texture.mipLevelCount, type: L },
    offset,
    hashInputs: [stage, format, samplePoints, mode, minFilter, L, offset]
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
.filter((t) => isDepthTextureFormat(t.format))
// MAINTENANCE_TODO: Remove when support for depth24plus, depth24plus-stencil8, and depth32float-stencil8 is added.
.filter((t) => isEncodableTextureFormat(t.format)).
combine('filt', ['nearest', 'linear']).
combine('mode', kShortAddressModes).
combine('offset', [false, true]).
beginSubcases().
combine('samplePoints', kSamplePointMethods).
combine('A', ['i32', 'u32']).
combine('L', ['i32', 'u32'])
).
beforeAllSubcases((t) =>
skipIfTextureFormatNotSupportedNotAvailableOrNotFilterable(t, t.params.format)
).
fn(async (t) => {
  const { format, stage, samplePoints, mode, filt: minFilter, A, L, offset } = t.params;

  // We want at least 4 blocks or something wide enough for 3 mip levels.
  const [width, height] = chooseTextureSize({ minSize: 8, minBlocks: 4, format });
  const descriptor = {
    format,
    size: { width, height },
    mipLevelCount: 3,
    usage: GPUTextureUsage.COPY_DST | GPUTextureUsage.TEXTURE_BINDING,
    ...(t.isCompatibility && { textureBindingViewDimension: '2d-array' })
  };
  const { texels, texture } = await createTextureWithRandomDataAndGetTexels(t, descriptor);
  const sampler = {
    addressModeU: kShortAddressModeToAddressMode[mode],
    addressModeV: kShortAddressModeToAddressMode[mode],
    minFilter,
    magFilter: minFilter,
    mipmapFilter: minFilter
  };

  const calls = generateTextureBuiltinInputs2D(50, {
    method: samplePoints,
    sampler,
    descriptor,
    arrayIndex: { num: texture.depthOrArrayLayers, type: A },
    mipLevel: { num: texture.mipLevelCount, type: L },
    offset,
    hashInputs: [stage, format, samplePoints, mode, minFilter, L, A, offset]
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
.filter((t) => isDepthTextureFormat(t.format))
// MAINTENANCE_TODO: Remove when support for depth24plus, depth24plus-stencil8, and depth32float-stencil8 is added.
.filter((t) => isEncodableTextureFormat(t.format)).
combineWithParams([
{ viewDimension: 'cube' },
{ viewDimension: 'cube-array', A: 'i32' },
{ viewDimension: 'cube-array', A: 'u32' }]
).
combine('filt', ['nearest', 'linear']).
combine('mode', kShortAddressModes).
beginSubcases().
combine('samplePoints', kCubeSamplePointMethods).
combine('L', ['i32', 'u32'])
).
beforeAllSubcases((t) => {
  skipIfTextureFormatNotSupportedNotAvailableOrNotFilterable(t, t.params.format);
  t.skipIfTextureViewDimensionNotSupported(t.params.viewDimension);
}).
fn(async (t) => {
  const { format, stage, viewDimension, samplePoints, A, L, mode, filt: minFilter } = t.params;

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
    addressModeW: kShortAddressModeToAddressMode[mode],
    minFilter,
    magFilter: minFilter,
    mipmapFilter: minFilter
  };

  const calls = generateSamplePointsCube(50, {
    method: samplePoints,
    sampler,
    descriptor,
    mipLevel: { num: texture.mipLevelCount - 1, type: L },
    arrayIndex: A ? { num: texture.depthOrArrayLayers / 6, type: A } : undefined,
    hashInputs: [stage, format, viewDimension, samplePoints, mode, minFilter]
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