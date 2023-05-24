/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Execution tests for the 'textureGather' builtin function

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
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';

import { generateCoordBoundaries, generateOffsets } from './utils.js';

export const g = makeTestGroup(GPUTest);

g.test('sampled_2d_coords')
  .specURL('https://www.w3.org/TR/WGSL/#texturegather')
  .desc(
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
  )
  .paramsSubcasesOnly(u =>
    u
      .combine('T', ['f32-only', 'i32', 'u32'])
      .combine('S', ['clamp-to-edge', 'repeat', 'mirror-repeat'])
      .combine('C', ['i32', 'u32'])
      .combine('C_value', [-1, 0, 1, 2, 3, 4])
      .combine('coords', generateCoordBoundaries(2))
      .combine('offset', generateOffsets(2))
  )
  .unimplemented();

g.test('sampled_3d_coords')
  .specURL('https://www.w3.org/TR/WGSL/#texturegather')
  .desc(
    `
C: i32, u32
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
  )
  .paramsSubcasesOnly(u =>
    u
      .combine('T', ['f32-only', 'i32', 'u32'])
      .combine('S', ['clamp-to-edge', 'repeat', 'mirror-repeat'])
      .combine('C', ['i32', 'u32'])
      .combine('C_value', [-1, 0, 1, 2, 3, 4])
      .combine('coords', generateCoordBoundaries(3))
  )
  .unimplemented();

g.test('sampled_array_2d_coords')
  .specURL('https://www.w3.org/TR/WGSL/#texturegather')
  .desc(
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
  )
  .paramsSubcasesOnly(u =>
    u
      .combine('T', ['f32-only', 'i32', 'u32'])
      .combine('S', ['clamp-to-edge', 'repeat', 'mirror-repeat'])
      .combine('C', ['i32', 'u32'])
      .combine('C_value', [-1, 0, 1, 2, 3, 4])
      .combine('coords', generateCoordBoundaries(2))
      /* array_index not param'd as out-of-bounds is implementation specific */ .combine(
        'offset',
        generateOffsets(2)
      )
  )
  .unimplemented();

g.test('sampled_array_3d_coords')
  .specURL('https://www.w3.org/TR/WGSL/#texturegather')
  .desc(
    `
C: i32, u32
T: i32, u32, f32

fn textureGather(component: C, t: texture_cube_array<T>, s: sampler, coords: vec3<f32>, array_index: C) -> vec4<T>

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
  )
  .paramsSubcasesOnly(
    u =>
      u
        .combine('T', ['f32-only', 'i32', 'u32'])
        .combine('S', ['clamp-to-edge', 'repeat', 'mirror-repeat'])
        .combine('C', ['i32', 'u32'])
        .combine('C_value', [-1, 0, 1, 2, 3, 4])
        .combine('coords', generateCoordBoundaries(3))
    /* array_index not param'd as out-of-bounds is implementation specific */
  )
  .unimplemented();

g.test('depth_2d_coords')
  .specURL('https://www.w3.org/TR/WGSL/#texturegather')
  .desc(
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
  )
  .paramsSubcasesOnly(u =>
    u
      .combine('S', ['clamp-to-edge', 'repeat', 'mirror-repeat'])
      .combine('coords', generateCoordBoundaries(2))
      .combine('offset', generateOffsets(2))
  )
  .unimplemented();

g.test('depth_3d_coords')
  .specURL('https://www.w3.org/TR/WGSL/#texturegather')
  .desc(
    `
fn textureGather(t: texture_depth_cube, s: sampler, coords: vec3<f32>) -> vec4<f32>

Parameters:
 * t: The depth texture to read from
 * s: The sampler type
 * coords: The texture coordinates
`
  )
  .paramsSubcasesOnly(u =>
    u
      .combine('S', ['clamp-to-edge', 'repeat', 'mirror-repeat'])
      .combine('coords', generateCoordBoundaries(3))
  )
  .unimplemented();

g.test('depth_array_2d_coords')
  .specURL('https://www.w3.org/TR/WGSL/#texturegather')
  .desc(
    `
C: i32, u32

fn textureGather(t: texture_depth_2d_array, s: sampler, coords: vec2<f32>, array_index: C) -> vec4<f32>
fn textureGather(t: texture_depth_2d_array, s: sampler, coords: vec2<f32>, array_index: C, offset: vec2<i32>) -> vec4<f32>

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
  )
  .paramsSubcasesOnly(u =>
    u
      .combine('S', ['clamp-to-edge', 'repeat', 'mirror-repeat'])
      .combine('C', ['i32', 'u32'])
      .combine('coords', generateCoordBoundaries(2))
      /* array_index not param'd as out-of-bounds is implementation specific */ .combine(
        'offset',
        generateOffsets(2)
      )
  )
  .unimplemented();

g.test('depth_array_3d_coords')
  .specURL('https://www.w3.org/TR/WGSL/#texturegather')
  .desc(
    `
C: i32, u32

fn textureGather(t: texture_depth_cube_array, s: sampler, coords: vec3<f32>, array_index: C) -> vec4<f32>

Parameters:
 * t: The depth texture to read from
 * s: The sampler type
 * coords: The texture coordinates
 * array_index: The 0-based texture array index
`
  )
  .paramsSubcasesOnly(
    u =>
      u
        .combine('S', ['clamp-to-edge', 'repeat', 'mirror-repeat'])
        .combine('C', ['i32', 'u32'])
        .combine('coords', generateCoordBoundaries(3))
    /* array_index not param'd as out-of-bounds is implementation specific */
  )
  .unimplemented();
