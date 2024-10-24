/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Samples a texture.

- TODO: test cube maps with more than 1 mip level.
- TODO: test un-encodable formats.

note: uniformity validation is covered in src/webgpu/shader/validation/uniformity/uniformity.spec.ts
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import {
  isDepthTextureFormat,
  isEncodableTextureFormat,
  kCompressedTextureFormats,
  kDepthStencilFormats,
  kEncodableTextureFormats,
  textureDimensionAndFormatCompatible } from
'../../../../../format_info.js';
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

  generateTextureBuiltinInputs1D,
  skipIfNeedsFilteringAndIsUnfilterable } from
'./texture_utils.js';

const kTestableColorFormats = [...kEncodableTextureFormats, ...kCompressedTextureFormats];

export const g = makeTestGroup(TextureTestMixin(WGSLTextureSampleTest));

g.test('sampled_1d_coords').
specURL('https://www.w3.org/TR/WGSL/#texturesample').
desc(
  `
fn textureSample(t: texture_1d<f32>, s: sampler, coords: f32) -> vec4<f32>

Parameters:
 * t  The sampled, depth, or external texture to sample.
 * s  The sampler type.
 * coords The texture coordinates used for sampling.
`
).
params((u) =>
u.
combine('format', kTestableColorFormats).
filter((t) => textureDimensionAndFormatCompatible('1d', t.format)).
filter((t) => isPotentiallyFilterableAndFillable(t.format)).
combine('filt', ['nearest', 'linear']).
combine('modeU', kShortAddressModes).
beginSubcases().
combine('samplePoints', kSamplePointMethods)
).
beforeAllSubcases((t) =>
skipIfTextureFormatNotSupportedNotAvailableOrNotFilterable(t, t.params.format)
).
fn(async (t) => {
  const { format, samplePoints, modeU, filt: minFilter } = t.params;

  // We want at least 4 blocks or something wide enough for 3 mip levels.
  const size = chooseTextureSize({ minSize: 8, minBlocks: 4, format, viewDimension: '1d' });

  const descriptor = {
    format,
    dimension: '1d',
    size,
    usage: GPUTextureUsage.COPY_DST | GPUTextureUsage.TEXTURE_BINDING
  };
  const { texels, texture } = await createTextureWithRandomDataAndGetTexels(t, descriptor);
  const sampler = {
    addressModeU: kShortAddressModeToAddressMode[modeU],
    minFilter,
    magFilter: minFilter
  };

  const calls = generateTextureBuiltinInputs1D(50, {
    sampler,
    method: samplePoints,
    descriptor,
    derivatives: true,
    hashInputs: [format, samplePoints, modeU, minFilter]
  }).map(({ coords, derivativeMult }) => {
    return {
      builtin: 'textureSample',
      coordType: 'f',
      coords,
      derivativeMult
    };
  });
  const viewDescriptor = {};
  const textureType = 'texture_1d<f32>';
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

g.test('sampled_2d_coords').
specURL('https://www.w3.org/TR/WGSL/#texturesample').
desc(
  `
fn textureSample(t: texture_2d<f32>, s: sampler, coords: vec2<f32>) -> vec4<f32>
fn textureSample(t: texture_2d<f32>, s: sampler, coords: vec2<f32>, offset: vec2<i32>) -> vec4<f32>

Parameters:
 * t  The sampled, depth, or external texture to sample.
 * s  The sampler type.
 * coords The texture coordinates used for sampling.
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
combine('format', kTestableColorFormats).
filter((t) => isPotentiallyFilterableAndFillable(t.format)).
combine('filt', ['nearest', 'linear']).
combine('modeU', kShortAddressModes).
combine('modeV', kShortAddressModes).
combine('offset', [false, true]).
beginSubcases().
combine('samplePoints', kSamplePointMethods)
).
beforeAllSubcases((t) => {
  skipIfTextureFormatNotSupportedNotAvailableOrNotFilterable(t, t.params.format);
}).
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
    derivatives: true,
    offset: true,
    hashInputs: [format, samplePoints, modeU, modeV, minFilter, offset]
  }).map(({ coords, derivativeMult, offset }) => {
    return {
      builtin: 'textureSample',
      coordType: 'f',
      coords,
      derivativeMult,
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
specURL('https://www.w3.org/TR/WGSL/#texturesample').
desc(
  `
fn textureSample(t: texture_3d<f32>, s: sampler, coords: vec3<f32>) -> vec4<f32>
fn textureSample(t: texture_3d<f32>, s: sampler, coords: vec3<f32>, offset: vec3<i32>) -> vec4<f32>
fn textureSample(t: texture_cube<f32>, s: sampler, coords: vec3<f32>) -> vec4<f32>

Parameters:
 * t  The sampled, depth, or external texture to sample.
 * s  The sampler type.
 * coords The texture coordinates used for sampling.
 * offset
    * The optional texel offset applied to the unnormalized texture coordinate before sampling the texture.
    * This offset is applied before applying any texture wrapping modes.
    * The offset expression must be a creation-time expression (e.g. vec2<i32>(1, 2)).
    * Each offset component must be at least -8 and at most 7.
      Values outside of this range will result in a shader-creation error.

* TODO: test 3d compressed textures formats. Just remove the filter below 'viewDimension'
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
    usage: GPUTextureUsage.COPY_DST | GPUTextureUsage.TEXTURE_BINDING,
    // MAINTENANCE_TODO: test derivatives with cubemaps by just always setting this to 3.
    mipLevelCount: viewDimension === '3d' ? 3 : 1
  };
  const { texels, texture } = await createTextureWithRandomDataAndGetTexels(t, descriptor);
  const sampler = {
    addressModeU: kShortAddressModeToAddressMode[modeU],
    addressModeV: kShortAddressModeToAddressMode[modeV],
    addressModeW: kShortAddressModeToAddressMode[modeW],
    minFilter,
    magFilter: minFilter,
    mipmapFilter: minFilter
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
    derivatives: true,
    hashInputs
  }) :
  generateSamplePointsCube(50, {
    method: samplePoints,
    sampler,
    descriptor,
    derivatives: true,
    hashInputs
  })).
  map(({ coords, derivativeMult, offset }) => {
    return {
      builtin: 'textureSample',
      coordType: 'f',
      coords,
      derivativeMult,
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

g.test('depth_2d_coords').
specURL('https://www.w3.org/TR/WGSL/#texturesample').
desc(
  `
fn textureSample(t: texture_depth_2d, s: sampler, coords: vec2<f32>) -> f32
fn textureSample(t: texture_depth_2d, s: sampler, coords: vec2<f32>, offset: vec2<i32>) -> f32

Parameters:
 * t  The sampled, depth, or external texture to sample.
 * s  The sampler type.
 * coords The texture coordinates used for sampling.
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
combine('samplePoints', kSamplePointMethods)
).
fn(async (t) => {
  const { format, samplePoints, modeU, modeV, filt: minFilter, offset } = t.params;

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
    derivatives: true,
    offset,
    hashInputs: [format, samplePoints, modeU, modeV, minFilter, offset]
  }).map(({ coords, derivativeMult, offset }) => {
    return {
      builtin: 'textureSample',
      coordType: 'f',
      coords,
      derivativeMult,
      offset
    };
  });

  const viewDescriptor = {};
  const textureType = 'texture_depth_2d';
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

g.test('sampled_array_2d_coords').
specURL('https://www.w3.org/TR/WGSL/#texturesample').
desc(
  `
A is i32 or u32

fn textureSample(t: texture_2d_array<f32>, s: sampler, coords: vec2<f32>, array_index: A) -> vec4<f32>
fn textureSample(t: texture_2d_array<f32>, s: sampler, coords: vec2<f32>, array_index: A, offset: vec2<i32>) -> vec4<f32>

Parameters:
 * t  The sampled, depth, or external texture to sample.
 * s  The sampler type.
 * coords The texture coordinates used for sampling.
 * array_index The 0-based texture array index to sample.
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
    derivatives: true,
    arrayIndex: { num: texture.depthOrArrayLayers, type: A },
    offset,
    hashInputs: [format, samplePoints, A, modeU, modeV, minFilter, offset]
  }).map(({ coords, derivativeMult, arrayIndex, offset }) => {
    return {
      builtin: 'textureSample',
      coordType: 'f',
      coords,
      derivativeMult,
      arrayIndex,
      arrayIndexType: A === 'i32' ? 'i' : 'u',
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

g.test('sampled_array_3d_coords').
specURL('https://www.w3.org/TR/WGSL/#texturesample').
desc(
  `
A is i32 or u32

fn textureSample(t: texture_cube_array<f32>, s: sampler, coords: vec3<f32>, array_index: A) -> vec4<f32>

Parameters:
 * t  The sampled, depth, or external texture to sample.
 * s  The sampler type.
 * coords The texture coordinates used for sampling.
 * array_index The 0-based texture array index to sample.
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
    // MAINTENANCE_TODO: test derivatives with cubemaps by setting this to 3.
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
    derivatives: true,
    arrayIndex: { num: texture.depthOrArrayLayers / 6, type: A },
    hashInputs: [format, viewDimension, A, samplePoints, mode, minFilter]
  }).map(({ coords, derivativeMult, arrayIndex }) => {
    return {
      builtin: 'textureSample',
      coordType: 'f',
      coords,
      derivativeMult,
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

g.test('depth_3d_coords').
specURL('https://www.w3.org/TR/WGSL/#texturesample').
desc(
  `
fn textureSample(t: texture_depth_cube, s: sampler, coords: vec3<f32>) -> f32

Parameters:
 * t  The sampled, depth, or external texture to sample.
 * s  The sampler type.
 * coords The texture coordinates used for sampling.
`
).
params((u) =>
u.
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
combine('samplePoints', kCubeSamplePointMethods)
).
beforeAllSubcases((t) => {
  t.skipIfTextureViewDimensionNotSupported(t.params.viewDimension);
}).
fn(async (t) => {
  const { format, viewDimension, samplePoints, A, mode, filt: minFilter } = t.params;

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
    // MAINTENANCE_TODO: test derivatives with cubemaps by setting this to 3.
    mipLevelCount: 1,
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
    derivatives: true,
    arrayIndex: A ? { num: texture.depthOrArrayLayers / 6, type: A } : undefined,
    hashInputs: [format, viewDimension, samplePoints, mode, minFilter]
  }).map(({ coords, derivativeMult, arrayIndex }) => {
    return {
      builtin: 'textureSample',
      coordType: 'f',
      coords,
      derivativeMult,
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

g.test('depth_array_2d_coords').
specURL('https://www.w3.org/TR/WGSL/#texturesample').
desc(
  `
A is i32 or u32

fn textureSample(t: texture_depth_2d_array, s: sampler, coords: vec2<f32>, array_index: A) -> f32
fn textureSample(t: texture_depth_2d_array, s: sampler, coords: vec2<f32>, array_index: A, offset: vec2<i32>) -> f32

Parameters:
 * t  The sampled, depth, or external texture to sample.
 * s  The sampler type.
 * coords The texture coordinates used for sampling.
 * array_index The 0-based texture array index to sample.
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
combine('mode', kShortAddressModes).
combine('offset', [false, true]).
beginSubcases().
combine('samplePoints', kSamplePointMethods).
combine('A', ['i32', 'u32']).
combine('L', ['i32', 'u32'])
).
fn(async (t) => {
  const { format, samplePoints, mode, filt: minFilter, A, L, offset } = t.params;

  // We want at least 4 blocks or something wide enough for 3 mip levels.
  const [width, height] = chooseTextureSize({ minSize: 8, minBlocks: 4, format });
  const descriptor = {
    format,
    size: { width, height },
    usage: GPUTextureUsage.COPY_DST | GPUTextureUsage.TEXTURE_BINDING,
    mipLevelCount: 3,
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
    derivatives: true,
    arrayIndex: { num: texture.depthOrArrayLayers, type: A },
    offset,
    hashInputs: [format, samplePoints, mode, minFilter, L, A, offset]
  }).map(({ coords, derivativeMult, arrayIndex, offset }) => {
    return {
      builtin: 'textureSample',
      coordType: 'f',
      coords,
      derivativeMult,
      arrayIndex,
      arrayIndexType: A === 'i32' ? 'i' : 'u',
      offset
    };
  });
  const textureType = 'texture_depth_2d_array';
  const viewDescriptor = { dimension: '2d-array' };
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

g.test('depth_array_3d_coords').
specURL('https://www.w3.org/TR/WGSL/#texturesample').
desc(
  `
A is i32 or u32

fn textureSample(t: texture_depth_cube_array, s: sampler, coords: vec3<f32>, array_index: A) -> f32

Parameters:
 * t  The sampled, depth, or external texture to sample.
 * s  The sampler type.
 * coords The texture coordinates used for sampling.
 * array_index The 0-based texture array index to sample.
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
combine('A', ['i32', 'u32'])
).
beforeAllSubcases((t) => {
  t.skipIfTextureViewDimensionNotSupported('cube-array');
}).
fn(async (t) => {
  const { format, samplePoints, A, mode, filt: minFilter } = t.params;

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
    // MAINTENANCE_TODO: test derivatives with cubemaps by setting this to 3.
    mipLevelCount: 1,
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
    derivatives: true,
    arrayIndex: A ? { num: texture.depthOrArrayLayers / 6, type: A } : undefined,
    hashInputs: [format, viewDimension, samplePoints, mode, minFilter]
  }).map(({ coords, derivativeMult, arrayIndex }) => {
    return {
      builtin: 'textureSample',
      coordType: 'f',
      coords,
      derivativeMult,
      arrayIndex,
      arrayIndexType: A ? A === 'i32' ? 'i' : 'u' : undefined
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