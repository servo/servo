/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution tests for the 'textureGather' builtin function

- TODO: Test un-encodable formats.

A texture gather operation reads from a 2D, 2D array, cube, or cube array texture, computing a four-component vector as follows:
 * Find the four texels that would be used in a sampling operation with linear filtering, from mip level 0:
   - Use the specified coordinate, array index (when present), and offset (when present).
   - The texels are adjacent, forming a square, when considering their texture space coordinates (u,v).
   - Selected texels at the texture edge, cube face edge, or cube corners are handled as in ordinary texture sampling.
 * For each texel, read one channel and convert it into a scalar value.
   - For non-depth textures, a zero-based component parameter specifies the channel to use.
     * If the texture format supports the specified channel, i.e. has more than component channels:
       - Yield scalar value v[component] when the texel value is v.
     * Otherwise:
       - Yield 0.0 when component is 1 or 2.
       - Yield 1.0 when component is 3 (the alpha channel).
   - For depth textures, yield the texel value. (Depth textures only have one channel.)
 * Yield the four-component vector, arranging scalars produced by the previous step into components according to the relative coordinates of the texels, as follows:
   - Result component  Relative texel coordinate
      x (umin,vmax)
      y (umax,vmax)
      z (umax,vmin)
      w (umin,vmin)
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
  isFillable,
  kCubeSamplePointMethods,
  kSamplePointMethods,
  kShortAddressModes,
  kShortAddressModeToAddressMode,
  kShortShaderStages,
  skipIfNeedsFilteringAndIsUnfilterableOrSelectDevice,



  WGSLTextureSampleTest } from
'./texture_utils.js';

const kTestableColorFormats = [...kEncodableTextureFormats, ...kCompressedTextureFormats];

export const g = makeTestGroup(WGSLTextureSampleTest);

g.test('sampled_2d_coords').
specURL('https://www.w3.org/TR/WGSL/#texturegather').
desc(
  `
C: i32, u32
T: i32, u32, f32

fn textureGather(component: C, t: texture_2d<T>, s: sampler, coords: vec2<f32>) -> vec4<T>
fn textureGather(component: C, t: texture_2d<T>, s: sampler, coords: vec2<f32>, offset: vec2<i32>) -> vec4<T>

Parameters:
 * component:
    - The index of the channel to read from the selected texels.
    - When provided, the component expression must a creation-time expression (e.g. 1).
    - Its value must be at least 0 and at most 3. Values outside of this range will result in a shader-creation error.
 * t: The sampled texture to read from
 * s: The sampler type
 * coords: The texture coordinates
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
combine('stage', kShortShaderStages).
combine('format', kTestableColorFormats).
filter((t) => isFillable(t.format)).
combine('filt', ['nearest', 'linear']).
combine('modeU', kShortAddressModes).
combine('modeV', kShortAddressModes).
combine('offset', [false, true]).
beginSubcases().
combine('C', ['i32', 'u32']).
combine('samplePoints', kSamplePointMethods)
).
beforeAllSubcases((t) => {
  t.skipIfTextureFormatNotSupported(t.params.format);
  skipIfNeedsFilteringAndIsUnfilterableOrSelectDevice(t, t.params.filt, t.params.format);
}).
fn(async (t) => {
  const { format, C, samplePoints, stage, modeU, modeV, filt: minFilter, offset } = t.params;

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
    textureBuiltin: 'textureGather',
    sampler,
    descriptor,
    offset,
    component: true,
    hashInputs: [stage, format, C, samplePoints, modeU, modeV, minFilter, offset]
  }).map(({ coords, component, offset }) => {
    return {
      builtin: 'textureGather',
      coordType: 'f',
      coords,
      component,
      componentType: C === 'i32' ? 'i' : 'u',
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

g.test('sampled_3d_coords').
specURL('https://www.w3.org/TR/WGSL/#texturegather').
desc(
  `
T: i32, u32, f32

fn textureGather(component: C, t: texture_cube<T>, s: sampler, coords: vec3<f32>) -> vec4<T>

Parameters:
 * component:
    - The index of the channel to read from the selected texels.
    - When provided, the component expression must a creation-time expression (e.g. 1).
    - Its value must be at least 0 and at most 3. Values outside of this range will result in a shader-creation error.
 * t: The sampled texture to read from
 * s: The sampler type
 * coords: The texture coordinates
`
).
params((u) =>
u.
combine('stage', kShortShaderStages).
combine('format', kTestableColorFormats).
filter((t) => isFillable(t.format)).
combine('filt', ['nearest', 'linear']).
combine('mode', kShortAddressModes).
beginSubcases().
combine('C', ['i32', 'u32']).
combine('samplePoints', kCubeSamplePointMethods)
).
beforeAllSubcases((t) => {
  t.skipIfTextureFormatNotSupported(t.params.format);
  skipIfNeedsFilteringAndIsUnfilterableOrSelectDevice(t, t.params.filt, t.params.format);
}).
fn(async (t) => {
  const { format, C, stage, samplePoints, mode, filt: minFilter } = t.params;

  const viewDimension = 'cube';
  const [width, height] = chooseTextureSize({ minSize: 8, minBlocks: 2, format, viewDimension });
  const depthOrArrayLayers = 6;

  const descriptor = {
    format,
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

  const calls = generateSamplePointsCube(50, {
    method: samplePoints,
    sampler,
    descriptor,
    component: true,
    textureBuiltin: 'textureGather',
    hashInputs: [stage, format, C, samplePoints, mode, minFilter]
  }).map(({ coords, component }) => {
    return {
      builtin: 'textureGather',
      component,
      componentType: C === 'i32' ? 'i' : 'u',
      coordType: 'f',
      coords
    };
  });
  const viewDescriptor = {
    dimension: viewDimension
  };
  const textureType = appendComponentTypeForFormatToTextureType('texture_cube', format);
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
specURL('https://www.w3.org/TR/WGSL/#texturegather').
desc(
  `
C: i32, u32
T: i32, u32, f32

fn textureGather(component: C, t: texture_2d_array<T>, s: sampler, coords: vec2<f32>, array_index: C) -> vec4<T>
fn textureGather(component: C, t: texture_2d_array<T>, s: sampler, coords: vec2<f32>, array_index: C, offset: vec2<i32>) -> vec4<T>

Parameters:
 * component:
    - The index of the channel to read from the selected texels.
    - When provided, the component expression must a creation-time expression (e.g. 1).
    - Its value must be at least 0 and at most 3. Values outside of this range will result in a shader-creation error.
 * t: The sampled texture to read from
 * s: The sampler type
 * coords: The texture coordinates
 * array_index: The 0-based texture array index
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
combine('stage', kShortShaderStages).
combine('format', kTestableColorFormats).
filter((t) => isFillable(t.format)).
combine('filt', ['nearest', 'linear']).
combine('modeU', kShortAddressModes).
combine('modeV', kShortAddressModes).
combine('offset', [false, true]).
beginSubcases().
combine('samplePoints', kSamplePointMethods).
combine('C', ['i32', 'u32']).
combine('A', ['i32', 'u32'])
).
beforeAllSubcases((t) => {
  t.skipIfTextureFormatNotSupported(t.params.format);
  skipIfNeedsFilteringAndIsUnfilterableOrSelectDevice(t, t.params.filt, t.params.format);
}).
fn(async (t) => {
  const { format, stage, samplePoints, C, A, modeU, modeV, filt: minFilter, offset } = t.params;

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
    textureBuiltin: 'textureGather',
    sampler,
    descriptor,
    arrayIndex: { num: texture.depthOrArrayLayers, type: A },
    offset,
    component: true,
    hashInputs: [stage, format, samplePoints, C, A, modeU, modeV, minFilter, offset]
  }).map(({ coords, component, arrayIndex, offset }) => {
    return {
      builtin: 'textureGather',
      component,
      componentType: C === 'i32' ? 'i' : 'u',
      coordType: 'f',
      coords,
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

g.test('sampled_array_3d_coords').
specURL('https://www.w3.org/TR/WGSL/#texturegather').
desc(
  `
C: i32, u32
T: i32, u32, f32
A: i32, u32

fn textureGather(component: C, t: texture_cube_array<T>, s: sampler, coords: vec3<f32>, array_index: A) -> vec4<T>

Parameters:
 * component:
    - The index of the channel to read from the selected texels.
    - When provided, the component expression must a creation-time expression (e.g. 1).
    - Its value must be at least 0 and at most 3. Values outside of this range will result in a shader-creation error.
 * t: The sampled texture to read from
 * s: The sampler type
 * coords: The texture coordinates
 * array_index: The 0-based texture array index
`
).
params((u) =>
u.
combine('stage', kShortShaderStages).
combine('format', kTestableColorFormats).
filter((t) => isFillable(t.format)).
combine('filt', ['nearest', 'linear']).
combine('mode', kShortAddressModes).
beginSubcases().
combine('samplePoints', kCubeSamplePointMethods).
combine('C', ['i32', 'u32']).
combine('A', ['i32', 'u32'])
).
beforeAllSubcases((t) => {
  t.skipIfTextureFormatNotSupported(t.params.format);
  t.skipIfTextureViewDimensionNotSupported('cube-array');
  skipIfNeedsFilteringAndIsUnfilterableOrSelectDevice(t, t.params.filt, t.params.format);
}).
fn(async (t) => {
  const { format, C, A, stage, samplePoints, mode, filt: minFilter } = t.params;

  const viewDimension = 'cube-array';
  const size = chooseTextureSize({ minSize: 8, minBlocks: 2, format, viewDimension });

  const descriptor = {
    format,
    ...(t.isCompatibility && { textureBindingViewDimension: viewDimension }),
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
    component: true,
    textureBuiltin: 'textureGather',
    arrayIndex: { num: texture.depthOrArrayLayers / 6, type: A },
    hashInputs: [stage, format, C, samplePoints, mode, minFilter]
  }).map(({ coords, component, arrayIndex }) => {
    return {
      builtin: 'textureGather',
      component,
      componentType: C === 'i32' ? 'i' : 'u',
      arrayIndex,
      arrayIndexType: A === 'i32' ? 'i' : 'u',
      coordType: 'f',
      coords
    };
  });
  const viewDescriptor = {
    dimension: viewDimension
  };
  const textureType = appendComponentTypeForFormatToTextureType('texture_cube_array', format);
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
specURL('https://www.w3.org/TR/WGSL/#texturegather').
desc(
  `
fn textureGather(t: texture_depth_2d, s: sampler, coords: vec2<f32>) -> vec4<f32>
fn textureGather(t: texture_depth_2d, s: sampler, coords: vec2<f32>, offset: vec2<i32>) -> vec4<f32>

Parameters:
 * t: The depth texture to read from
 * s: The sampler type
 * coords: The texture coordinates
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
combine('stage', kShortShaderStages).
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
  const { format, stage, samplePoints, modeU, modeV, filt: minFilter, offset } = t.params;

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
    textureBuiltin: 'textureGather',
    sampler,
    descriptor,
    offset,
    hashInputs: [stage, format, samplePoints, modeU, modeV, minFilter, offset]
  }).map(({ coords, offset }) => {
    return {
      builtin: 'textureGather',
      coordType: 'f',
      coords,
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
specURL('https://www.w3.org/TR/WGSL/#texturegather').
desc(
  `
fn textureGather(t: texture_depth_cube, s: sampler, coords: vec3<f32>) -> vec4<f32>

Parameters:
 * t: The depth texture to read from
 * s: The sampler type
 * coords: The texture coordinates
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
beginSubcases().
combine('samplePoints', kCubeSamplePointMethods)
).
fn(async (t) => {
  const { format, stage, samplePoints, mode, filt: minFilter } = t.params;

  const viewDimension = 'cube';
  const [width, height] = chooseTextureSize({ minSize: 8, minBlocks: 2, format, viewDimension });
  const depthOrArrayLayers = 6;

  const descriptor = {
    format,
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

  const calls = generateSamplePointsCube(50, {
    method: samplePoints,
    sampler,
    descriptor,
    textureBuiltin: 'textureGather',
    hashInputs: [stage, format, samplePoints, mode, minFilter]
  }).map(({ coords, component }) => {
    return {
      builtin: 'textureGather',
      coordType: 'f',
      coords
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
specURL('https://www.w3.org/TR/WGSL/#texturegather').
desc(
  `
A: i32, u32

fn textureGather(t: texture_depth_2d_array, s: sampler, coords: vec2<f32>, array_index: A) -> vec4<f32>
fn textureGather(t: texture_depth_2d_array, s: sampler, coords: vec2<f32>, array_index: A, offset: vec2<i32>) -> vec4<f32>

Parameters:
 * t: The depth texture to read from
 * s: The sampler type
 * coords: The texture coordinates
 * array_index: The 0-based texture array index
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
combine('stage', kShortShaderStages).
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
combine('A', ['i32', 'u32'])
).
beforeAllSubcases((t) => {
  t.skipIfTextureFormatNotSupported(t.params.format);
  skipIfNeedsFilteringAndIsUnfilterableOrSelectDevice(t, t.params.filt, t.params.format);
}).
fn(async (t) => {
  const { format, stage, samplePoints, A, modeU, modeV, filt: minFilter, offset } = t.params;

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
    textureBuiltin: 'textureGather',
    sampler,
    descriptor,
    arrayIndex: { num: texture.depthOrArrayLayers, type: A },
    offset,
    hashInputs: [stage, format, samplePoints, A, modeU, modeV, minFilter, offset]
  }).map(({ coords, arrayIndex, offset }) => {
    return {
      builtin: 'textureGather',
      coordType: 'f',
      coords,
      arrayIndex,
      arrayIndexType: A === 'i32' ? 'i' : 'u',
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

g.test('depth_array_3d_coords').
specURL('https://www.w3.org/TR/WGSL/#texturegather').
desc(
  `
A: i32, u32

fn textureGather(t: texture_depth_cube_array, s: sampler, coords: vec3<f32>, array_index: A) -> vec4<f32>

Parameters:
 * t: The depth texture to read from
 * s: The sampler type
 * coords: The texture coordinates
 * array_index: The 0-based texture array index
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
beginSubcases().
combine('samplePoints', kCubeSamplePointMethods).
combine('A', ['i32', 'u32'])
).
beforeAllSubcases((t) => {
  t.skipIfTextureViewDimensionNotSupported('cube-array');
}).
fn(async (t) => {
  const { format, A, stage, samplePoints, mode, filt: minFilter } = t.params;

  const viewDimension = 'cube-array';
  const size = chooseTextureSize({ minSize: 8, minBlocks: 2, format, viewDimension });

  const descriptor = {
    format,
    ...(t.isCompatibility && { textureBindingViewDimension: viewDimension }),
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
    textureBuiltin: 'textureGather',
    arrayIndex: { num: texture.depthOrArrayLayers / 6, type: A },
    hashInputs: [stage, format, samplePoints, mode, minFilter]
  }).map(({ coords, arrayIndex }) => {
    return {
      builtin: 'textureGather',
      arrayIndex,
      arrayIndexType: A === 'i32' ? 'i' : 'u',
      coordType: 'f',
      coords
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