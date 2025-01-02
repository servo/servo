/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Samples a depth texture and compares the sampled depth values against a reference value.

- TODO: test cube maps with more than 1 mip level.
- TODO: test un-encodable formats.
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { kCompareFunctions } from '../../../../../capability_info.js';
import {
  isDepthTextureFormat,
  isEncodableTextureFormat,
  kDepthStencilFormats } from
'../../../../../format_info.js';

import {
  checkCallResults,
  chooseTextureSize,
  createTextureWithRandomDataAndGetTexels,
  doTextureCalls,
  generateSamplePointsCube,
  generateTextureBuiltinInputs2D,
  kCubeSamplePointMethods,
  kSamplePointMethods,
  kShortAddressModes,
  kShortAddressModeToAddressMode,
  makeRandomDepthComparisonTexelGenerator,



  WGSLTextureSampleTest } from
'./texture_utils.js';

export const g = makeTestGroup(WGSLTextureSampleTest);

g.test('2d_coords').
specURL('https://www.w3.org/TR/WGSL/#texturesamplecompare').
desc(
  `
fn textureSampleCompare(t: texture_depth_2d, s: sampler_comparison, coords: vec2<f32>, depth_ref: f32) -> f32
fn textureSampleCompare(t: texture_depth_2d, s: sampler_comparison, coords: vec2<f32>, depth_ref: f32, offset: vec2<i32>) -> f32

Parameters:
 * t  The depth texture to sample.
 * s  The sampler_comparison type.
 * coords The texture coordinates used for sampling.
 * depth_ref The reference value to compare the sampled depth value against.
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
combine('format', kDepthStencilFormats)
// filter out stencil only formats
.filter((t) => isDepthTextureFormat(t.format))
// MAINTENANCE_TODO: Remove when support for depth24plus, depth24plus-stencil8, and depth32float-stencil8 is added.
.filter((t) => isEncodableTextureFormat(t.format)).
combine('filt', ['nearest', 'linear']).
combine('modeU', kShortAddressModes).
combine('modeV', kShortAddressModes).
combine('offset', [false, true]).
beginSubcases().
combine('samplePoints', kSamplePointMethods).
combine('compare', kCompareFunctions)
).
fn(async (t) => {
  const { format, samplePoints, modeU, modeV, filt: minFilter, compare, offset } = t.params;

  const size = chooseTextureSize({ minSize: 16, minBlocks: 4, format });

  const descriptor = {
    format,
    size,
    usage: GPUTextureUsage.COPY_DST | GPUTextureUsage.TEXTURE_BINDING,
    mipLevelCount: 3
  };
  const { texels, texture } = await createTextureWithRandomDataAndGetTexels(t, descriptor, {
    generator: makeRandomDepthComparisonTexelGenerator(descriptor, compare)
  });
  const sampler = {
    addressModeU: kShortAddressModeToAddressMode[modeU],
    addressModeV: kShortAddressModeToAddressMode[modeV],
    compare,
    minFilter,
    magFilter: minFilter,
    mipmapFilter: minFilter
  };

  const calls = generateTextureBuiltinInputs2D(50, {
    method: samplePoints,
    textureBuiltin: 'textureSampleCompare',
    sampler,
    descriptor,
    derivatives: true,
    depthRef: true,
    offset,
    hashInputs: [format, samplePoints, modeU, modeV, minFilter, offset]
  }).map(({ coords, derivativeMult, arrayIndex, depthRef, offset }) => {
    return {
      builtin: 'textureSampleCompare',
      coordType: 'f',
      coords,
      derivativeMult,
      depthRef,
      offset
    };
  });
  const textureType = 'texture_depth_2d';
  const viewDescriptor = {};
  const results = await doTextureCalls(
    t,
    texture,
    viewDescriptor,
    textureType,
    sampler,
    calls,
    'f'
  );
  const res = await checkCallResults(
    t,
    { texels, descriptor, viewDescriptor },
    textureType,
    sampler,
    calls,
    results,
    'f',
    texture
  );
  t.expectOK(res);
});

g.test('3d_coords').
specURL('https://www.w3.org/TR/WGSL/#texturesamplecompare').
desc(
  `
fn textureSampleCompare(t: texture_depth_cube, s: sampler_comparison, coords: vec3<f32>, depth_ref: f32) -> f32

Parameters:
 * t  The depth texture to sample.
 * s  The sampler_comparison type.
 * coords The texture coordinates used for sampling.
 * depth_ref The reference value to compare the sampled depth value against.
`
).
params((u) =>
u.
combine('format', kDepthStencilFormats)
// filter out stencil only formats
.filter((t) => isDepthTextureFormat(t.format))
// MAINTENANCE_TODO: Remove when support for depth24plus, depth24plus-stencil8, and depth32float-stencil8 is added.
.filter((t) => isEncodableTextureFormat(t.format)).
combine('filt', ['nearest', 'linear']).
combine('mode', kShortAddressModes).
beginSubcases().
combine('samplePoints', kCubeSamplePointMethods).
combine('compare', kCompareFunctions)
).
fn(async (t) => {
  const { format, samplePoints, mode, filt: minFilter, compare } = t.params;

  const viewDimension = 'cube';
  const size = chooseTextureSize({ minSize: 16, minBlocks: 2, format, viewDimension });

  const descriptor = {
    format,
    ...(t.isCompatibility && { textureBindingViewDimension: viewDimension }),
    size,
    usage: GPUTextureUsage.COPY_DST | GPUTextureUsage.TEXTURE_BINDING,
    // MAINTENANCE_TODO: change to 3 once derivatives with cube maps are supported
    mipLevelCount: 1
  };
  const { texels, texture } = await createTextureWithRandomDataAndGetTexels(t, descriptor, {
    generator: makeRandomDepthComparisonTexelGenerator(descriptor, compare)
  });
  const sampler = {
    addressModeU: kShortAddressModeToAddressMode[mode],
    addressModeV: kShortAddressModeToAddressMode[mode],
    addressModeW: kShortAddressModeToAddressMode[mode],
    compare,
    minFilter,
    magFilter: minFilter,
    mipmapFilter: minFilter
  };

  const calls = generateSamplePointsCube(50, {
    method: samplePoints,
    sampler,
    descriptor,
    derivatives: true,
    depthRef: true,
    textureBuiltin: 'textureSampleCompare',
    hashInputs: [format, samplePoints, mode, minFilter, compare]
  }).map(({ coords, derivativeMult, depthRef }) => {
    return {
      builtin: 'textureSampleCompare',
      coordType: 'f',
      coords,
      derivativeMult,
      depthRef
    };
  });
  const viewDescriptor = {
    dimension: viewDimension
  };
  const textureType = 'texture_depth_cube';
  const results = await doTextureCalls(
    t,
    texture,
    viewDescriptor,
    textureType,
    sampler,
    calls,
    'f'
  );
  const res = await checkCallResults(
    t,
    { texels, descriptor, viewDescriptor },
    textureType,
    sampler,
    calls,
    results,
    'f',
    texture
  );
  t.expectOK(res);
});

g.test('arrayed_2d_coords').
specURL('https://www.w3.org/TR/WGSL/#texturesamplecompare').
desc(
  `
A is i32 or u32

fn textureSampleCompare(t: texture_depth_2d_array, s: sampler_comparison, coords: vec2<f32>, array_index: A, depth_ref: f32) -> f32
fn textureSampleCompare(t: texture_depth_2d_array, s: sampler_comparison, coords: vec2<f32>, array_index: A, depth_ref: f32, offset: vec2<i32>) -> f32

Parameters:
 * t  The depth texture to sample.
 * s  The sampler_comparison type.
 * coords The texture coordinates used for sampling.
 * array_index: The 0-based texture array index to sample.
 * depth_ref The reference value to compare the sampled depth value against.
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
combine('format', kDepthStencilFormats)
// filter out stencil only formats
.filter((t) => isDepthTextureFormat(t.format))
// MAINTENANCE_TODO: Remove when support for depth24plus, depth24plus-stencil8, and depth32float-stencil8 is added.
.filter((t) => isEncodableTextureFormat(t.format)).
combine('filt', ['nearest', 'linear']).
combine('modeU', kShortAddressModes).
combine('modeV', kShortAddressModes).
combine('offset', [false, true]).
beginSubcases().
combine('samplePoints', kSamplePointMethods).
combine('A', ['i32', 'u32']).
combine('compare', kCompareFunctions)
).
beforeAllSubcases((t) => {
  t.skipIfTextureFormatNotSupported(t.params.format);
}).
fn(async (t) => {
  const { format, samplePoints, A, modeU, modeV, filt: minFilter, compare, offset } = t.params;

  const viewDimension = '2d-array';
  const size = chooseTextureSize({ minSize: 16, minBlocks: 4, format, viewDimension });

  const descriptor = {
    format,
    size,
    usage: GPUTextureUsage.COPY_DST | GPUTextureUsage.TEXTURE_BINDING,
    mipLevelCount: 3
  };
  const { texels, texture } = await createTextureWithRandomDataAndGetTexels(t, descriptor, {
    generator: makeRandomDepthComparisonTexelGenerator(descriptor, compare)
  });
  const sampler = {
    addressModeU: kShortAddressModeToAddressMode[modeU],
    addressModeV: kShortAddressModeToAddressMode[modeV],
    compare,
    minFilter,
    magFilter: minFilter,
    mipmapFilter: minFilter
  };

  const calls = generateTextureBuiltinInputs2D(50, {
    method: samplePoints,
    textureBuiltin: 'textureSampleCompare',
    sampler,
    descriptor,
    derivatives: true,
    arrayIndex: { num: texture.depthOrArrayLayers, type: A },
    depthRef: true,
    offset,
    hashInputs: [format, samplePoints, A, modeU, modeV, minFilter, offset]
  }).map(({ coords, derivativeMult, arrayIndex, depthRef, offset }) => {
    return {
      builtin: 'textureSampleCompare',
      coordType: 'f',
      coords,
      derivativeMult,
      arrayIndex,
      arrayIndexType: A === 'i32' ? 'i' : 'u',
      depthRef,
      offset
    };
  });
  const textureType = 'texture_depth_2d_array';
  const viewDescriptor = {};
  const results = await doTextureCalls(
    t,
    texture,
    viewDescriptor,
    textureType,
    sampler,
    calls,
    'f'
  );
  const res = await checkCallResults(
    t,
    { texels, descriptor, viewDescriptor },
    textureType,
    sampler,
    calls,
    results,
    'f',
    texture
  );
  t.expectOK(res);
});

g.test('arrayed_3d_coords').
specURL('https://www.w3.org/TR/WGSL/#texturesamplecompare').
desc(
  `
A is i32 or u32

fn textureSampleCompare(t: texture_depth_cube_array, s: sampler_comparison, coords: vec3<f32>, array_index: A, depth_ref: f32) -> f32

Parameters:
 * t  The depth texture to sample.
 * s  The sampler_comparison type.
 * coords The texture coordinates used for sampling.
 * array_index: The 0-based texture array index to sample.
 * depth_ref The reference value to compare the sampled depth value against.
`
).
params((u) =>
u.
combine('format', kDepthStencilFormats)
// filter out stencil only formats
.filter((t) => isDepthTextureFormat(t.format))
// MAINTENANCE_TODO: Remove when support for depth24plus, depth24plus-stencil8, and depth32float-stencil8 is added.
.filter((t) => isEncodableTextureFormat(t.format)).
combine('filt', ['nearest', 'linear']).
combine('mode', kShortAddressModes).
beginSubcases().
combine('samplePoints', kCubeSamplePointMethods).
combine('A', ['i32', 'u32']).
combine('compare', kCompareFunctions)
).
beforeAllSubcases((t) => {
  t.skipIfTextureViewDimensionNotSupported('cube-array');
}).
fn(async (t) => {
  const { format, A, samplePoints, mode, filt: minFilter, compare } = t.params;

  const viewDimension = 'cube-array';
  const size = chooseTextureSize({ minSize: 8, minBlocks: 2, format, viewDimension });

  const descriptor = {
    format,
    ...(t.isCompatibility && { textureBindingViewDimension: viewDimension }),
    size,
    usage: GPUTextureUsage.COPY_DST | GPUTextureUsage.TEXTURE_BINDING,
    // MAINTENANCE_TODO: change to 3 once derivatives with cube maps are supported
    mipLevelCount: 1
  };
  const { texels, texture } = await createTextureWithRandomDataAndGetTexels(t, descriptor, {
    generator: makeRandomDepthComparisonTexelGenerator(descriptor, compare)
  });
  const sampler = {
    addressModeU: kShortAddressModeToAddressMode[mode],
    addressModeV: kShortAddressModeToAddressMode[mode],
    addressModeW: kShortAddressModeToAddressMode[mode],
    compare,
    minFilter,
    magFilter: minFilter,
    mipmapFilter: minFilter
  };

  const calls = generateSamplePointsCube(50, {
    method: samplePoints,
    sampler,
    descriptor,
    derivatives: true,
    textureBuiltin: 'textureSampleCompare',
    arrayIndex: { num: texture.depthOrArrayLayers / 6, type: A },
    depthRef: true,
    hashInputs: [format, samplePoints, mode, minFilter]
  }).map(({ coords, derivativeMult, depthRef, arrayIndex }) => {
    return {
      builtin: 'textureSampleCompare',
      arrayIndex,
      arrayIndexType: A === 'i32' ? 'i' : 'u',
      coordType: 'f',
      coords,
      derivativeMult,
      depthRef
    };
  });
  const viewDescriptor = {
    dimension: viewDimension
  };
  const textureType = 'texture_depth_cube_array';
  const results = await doTextureCalls(
    t,
    texture,
    viewDescriptor,
    textureType,
    sampler,
    calls,
    'f'
  );
  const res = await checkCallResults(
    t,
    { texels, descriptor, viewDescriptor },
    textureType,
    sampler,
    calls,
    results,
    'f',
    texture
  );
  t.expectOK(res);
});