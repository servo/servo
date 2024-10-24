/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution tests for the 'textureSampleBias' builtin function

Samples a texture with a bias to the mip level.

- TODO: test cube maps with more than one mip level.
- TODO: Test un-encodable formats.
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { kCompressedTextureFormats, kEncodableTextureFormats } from '../../../../../format_info.js';
import { TextureTestMixin } from '../../../../../gpu_test.js';

import {



  generateTextureBuiltinInputs2D,
  generateTextureBuiltinInputs3D,
  kSamplePointMethods,
  kShortAddressModes,
  kShortAddressModeToAddressMode,
  doTextureCalls,
  checkCallResults,
  createTextureWithRandomDataAndGetTexels,
  generateSamplePointsCube,
  kCubeSamplePointMethods,

  chooseTextureSize,
  isPotentiallyFilterableAndFillable,
  skipIfTextureFormatNotSupportedNotAvailableOrNotFilterable,
  getTextureTypeForTextureViewDimension,
  WGSLTextureSampleTest,
  isSupportedViewFormatCombo,
  skipIfNeedsFilteringAndIsUnfilterable } from
'./texture_utils.js';

const kTestableColorFormats = [...kEncodableTextureFormats, ...kCompressedTextureFormats];

export const g = makeTestGroup(TextureTestMixin(WGSLTextureSampleTest));

g.test('sampled_2d_coords').
specURL('https://www.w3.org/TR/WGSL/#texturesamplebias').
desc(
  `
fn textureSampleBias(t: texture_2d<f32>, s: sampler, coords: vec2<f32>, bias: f32) -> vec4<f32>
fn textureSampleBias(t: texture_2d<f32>, s: sampler, coords: vec2<f32>, bias: f32, offset: vec2<i32>) -> vec4<f32>

Parameters:
 * t: The sampled texture to read from
 * s: The sampler type
 * coords: The texture coordinates
 * bias: The bias to apply to the mip level before sampling. bias must be between -16.0 and 15.99.
 * offset:
    - The optional texel offset applied to the unnormalized texture coordinate before sampling the texture.
      This offset is applied before applying any texture wrapping modes.
    - The offset expression must be a creation-time expression (e.g. vec2<i32>(1, 2)).
    - Each offset component must be at least -8 and at most 7.
      Values outside of this range will result in a shader-creation error.
`
).
params((u) =>
u.
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
  const { format, samplePoints, modeU, modeV, filt: minFilter, offset } = t.params;
  skipIfNeedsFilteringAndIsUnfilterable(t, minFilter, format);

  // We want at least 4 blocks or something wide enough for 3 mip levels.
  const [width, height] = chooseTextureSize({ minSize: 8, minBlocks: 4, format });

  const descriptor = {
    format,
    size: { width, height },
    usage: GPUTextureUsage.COPY_DST | GPUTextureUsage.TEXTURE_BINDING,
    mipLevelCount: 3
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
    sampler,
    method: samplePoints,
    descriptor,
    bias: true,
    offset,
    hashInputs: [format, samplePoints, modeU, modeV, minFilter, offset]
  }).map(({ coords, derivativeMult, offset, bias }) => {
    return {
      builtin: 'textureSampleBias',
      coordType: 'f',
      coords,
      derivativeMult,
      bias,
      offset
    };
  });
  const viewDescriptor = {};
  const textureType = 'texture_2d<f32>';
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

g.test('sampled_3d_coords').
specURL('https://www.w3.org/TR/WGSL/#texturesamplebias').
desc(
  `
fn textureSampleBias(t: texture_3d<f32>, s: sampler, coords: vec3<f32>, bias: f32) -> vec4<f32>
fn textureSampleBias(t: texture_3d<f32>, s: sampler, coords: vec3<f32>, bias: f32, offset: vec3<i32>) -> vec4<f32>
fn textureSampleBias(t: texture_cube<f32>, s: sampler, coords: vec3<f32>, bias: f32) -> vec4<f32>

Parameters:
 * t: The sampled texture to read from
 * s: The sampler type
 * coords: The texture coordinates
 * bias: The bias to apply to the mip level before sampling. bias must be between -16.0 and 15.99.
 * offset:
    - The optional texel offset applied to the unnormalized texture coordinate before sampling the texture.
      This offset is applied before applying any texture wrapping modes.
    - The offset expression must be a creation-time expression (e.g. vec2<i32>(1, 2)).
    - Each offset component must be at least -8 and at most 7.
      Values outside of this range will result in a shader-creation error.
`
).
params((u) =>
u.
combine('format', kTestableColorFormats).
filter((t) => isPotentiallyFilterableAndFillable(t.format)).
combine('dim', ['3d', 'cube']).
filter((t) => isSupportedViewFormatCombo(t.format, t.dim)).
combine('filt', ['nearest', 'linear']).
combine('modeU', kShortAddressModes).
combine('modeV', kShortAddressModes).
combine('modeW', kShortAddressModes).
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
    samplePoints,
    modeU,
    modeV,
    modeW,
    filt: minFilter,
    offset
  } = t.params;
  skipIfNeedsFilteringAndIsUnfilterable(t, minFilter, format);

  const size = chooseTextureSize({ minSize: 8, minBlocks: 2, format, viewDimension });
  const descriptor = {
    format,
    dimension: viewDimension === '3d' ? '3d' : '2d',
    ...(t.isCompatibility && { textureBindingViewDimension: viewDimension }),
    size,
    // MAINTENANCE_TODO: use 3 for cube maps when derivatives are supported for cube maps.
    mipLevelCount: viewDimension === '3d' ? 3 : 1,
    usage: GPUTextureUsage.COPY_DST | GPUTextureUsage.TEXTURE_BINDING
  };
  const { texels, texture } = await createTextureWithRandomDataAndGetTexels(t, descriptor);
  const sampler = {
    addressModeU: kShortAddressModeToAddressMode[modeU],
    addressModeV: kShortAddressModeToAddressMode[modeV],
    addressModeW: kShortAddressModeToAddressMode[modeW],
    minFilter,
    magFilter: minFilter
  };

  const hashInputs = [
  format,
  viewDimension,
  samplePoints,
  modeU,
  modeV,
  modeW,
  minFilter,
  offset];

  const calls = (
  viewDimension === '3d' ?
  generateTextureBuiltinInputs3D(50, {
    method: samplePoints,
    sampler,
    descriptor,
    bias: true,
    offset,
    hashInputs
  }) :
  generateSamplePointsCube(50, {
    method: samplePoints,
    sampler,
    descriptor,
    bias: true,
    hashInputs
  })).
  map(({ coords, derivativeMult, offset, bias }) => {
    return {
      builtin: 'textureSampleBias',
      coordType: 'f',
      coords,
      derivativeMult,
      bias,
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
specURL('https://www.w3.org/TR/WGSL/#texturesamplebias').
desc(
  `
A: i32, u32

fn textureSampleBias(t: texture_2d_array<f32>, s: sampler, coords: vec2<f32>, array_index: A, bias: f32) -> vec4<f32>
fn textureSampleBias(t: texture_2d_array<f32>, s: sampler, coords: vec2<f32>, array_index: A, bias: f32, offset: vec2<i32>) -> vec4<f32>

Parameters:
 * t: The sampled texture to read from
 * s: The sampler type
 * coords: The texture coordinates
 * array_index: The 0-based texture array index to sample.
 * bias: The bias to apply to the mip level before sampling. bias must be between -16.0 and 15.99.
 * offset:
    - The optional texel offset applied to the unnormalized texture coordinate before sampling the texture.
      This offset is applied before applying any texture wrapping modes.
    - The offset expression must be a creation-time expression (e.g. vec2<i32>(1, 2)).
    - Each offset component must be at least -8 and at most 7.
      Values outside of this range will result in a shader-creation error.
`
).
params((u) =>
u.
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
  const { format, samplePoints, A, modeU, modeV, filt: minFilter, offset } = t.params;
  skipIfNeedsFilteringAndIsUnfilterable(t, minFilter, format);

  // We want at least 4 blocks or something wide enough for 3 mip levels.
  const [width, height] = chooseTextureSize({ minSize: 8, minBlocks: 4, format });
  const depthOrArrayLayers = 4;

  const descriptor = {
    format,
    size: { width, height, depthOrArrayLayers },
    usage: GPUTextureUsage.COPY_DST | GPUTextureUsage.TEXTURE_BINDING,
    mipLevelCount: 3
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
    arrayIndex: { num: texture.depthOrArrayLayers, type: A },
    bias: true,
    offset,
    hashInputs: [format, samplePoints, A, modeU, modeV, minFilter, offset]
  }).map(({ coords, derivativeMult, arrayIndex, bias, offset }) => {
    return {
      builtin: 'textureSampleBias',
      coordType: 'f',
      coords,
      derivativeMult,
      arrayIndex,
      arrayIndexType: A === 'i32' ? 'i' : 'u',
      bias,
      offset
    };
  });
  const textureType = 'texture_2d_array<f32>';
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
specURL('https://www.w3.org/TR/WGSL/#texturesamplebias').
desc(
  `
A: i32, u32

fn textureSampleBias(t: texture_cube_array<f32>, s: sampler, coords: vec3<f32>, array_index: A, bias: f32) -> vec4<f32>

Parameters:
 * t: The sampled texture to read from
 * s: The sampler type
 * coords: The texture coordinates
 * array_index: The 0-based texture array index to sample.
 * bias: The bias to apply to the mip level before sampling. bias must be between -16.0 and 15.99.
 * offset:
    - The optional texel offset applied to the unnormalized texture coordinate before sampling the texture.
      This offset is applied before applying any texture wrapping modes.
    - The offset expression must be a creation-time expression (e.g. vec2<i32>(1, 2)).
    - Each offset component must be at least -8 and at most 7.
      Values outside of this range will result in a shader-creation error.
`
).
params((u) =>
u.
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
  const { format, samplePoints, A, mode, filt: minFilter } = t.params;
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
    // MAINTENANCE_TODO: use 3 for cube maps when derivatives are supported for cube maps.
    mipLevelCount: 1
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
    bias: true,
    arrayIndex: { num: texture.depthOrArrayLayers / 6, type: A },
    hashInputs: [format, viewDimension, A, samplePoints, mode, minFilter]
  }).map(({ coords, derivativeMult, arrayIndex, bias }) => {
    return {
      builtin: 'textureSampleBias',
      coordType: 'f',
      coords,
      derivativeMult,
      arrayIndex,
      arrayIndexType: A === 'i32' ? 'i' : 'u',
      bias
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