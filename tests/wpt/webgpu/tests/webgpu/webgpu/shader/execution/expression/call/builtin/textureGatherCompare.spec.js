/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution tests for the 'textureGatherCompare' builtin function

A texture gather compare operation performs a depth comparison on four texels in a depth texture and collects the results into a single vector, as follows:
 * Find the four texels that would be used in a depth sampling operation with linear filtering, from mip level 0:
   - Use the specified coordinate, array index (when present), and offset (when present).
   - The texels are adjacent, forming a square, when considering their texture space coordinates (u,v).
   - Selected texels at the texture edge, cube face edge, or cube corners are handled as in ordinary texture sampling.
 * For each texel, perform a comparison against the depth reference value, yielding a 0.0 or 1.0 value, as controlled by the comparison sampler parameters.
 * Yield the four-component vector where the components are the comparison results with the texels with relative texel coordinates as follows:

   Result component  Relative texel coordinate
    x                (umin,vmax)
    y                (umax,vmax)
    z                (umax,vmin)
    w                (umin,vmin)
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';

import { generateCoordBoundaries, generateOffsets } from './utils.js';

export const g = makeTestGroup(GPUTest);

g.test('array_2d_coords').
specURL('https://www.w3.org/TR/WGSL/#texturegathercompare').
desc(
  `
C: i32, u32

fn textureGatherCompare(t: texture_depth_2d_array, s: sampler_comparison, coords: vec2<f32>, array_index: C, depth_ref: f32) -> vec4<f32>
fn textureGatherCompare(t: texture_depth_2d_array, s: sampler_comparison, coords: vec2<f32>, array_index: C, depth_ref: f32, offset: vec2<i32>) -> vec4<f32>

Parameters:
 * t: The depth texture to read from
 * s: The sampler_comparison
 * coords: The texture coordinates
 * array_index: The 0-based array index.
 * depth_ref: The reference value to compare the sampled depth value against
 * offset:
    - The optional texel offset applied to the unnormalized texture coordinate before sampling the texture.
      This offset is applied before applying any texture wrapping modes.
    - The offset expression must be a creation-time expression (e.g. vec2<i32>(1, 2)).
    - Each offset component must be at least -8 and at most 7.
      Values outside of this range will result in a shader-creation error.
`
).
paramsSubcasesOnly((u) =>
u.
combine('S', ['clamp-to-edge', 'repeat', 'mirror-repeat']).
combine('C', ['i32', 'u32']).
combine('C_value', [-1, 0, 1, 2, 3, 4]).
combine('coords', generateCoordBoundaries(2)).
combine('depth_ref', [-1 /* smaller ref */, 0 /* equal ref */, 1 /* larger ref */]).
combine('offset', generateOffsets(2))
).
unimplemented();

g.test('array_3d_coords').
specURL('https://www.w3.org/TR/WGSL/#texturegathercompare').
desc(
  `
C: i32, u32

fn textureGatherCompare(t: texture_depth_cube_array, s: sampler_comparison, coords: vec3<f32>, array_index: C, depth_ref: f32) -> vec4<f32>

Parameters:
 * t: The depth texture to read from
 * s: The sampler_comparison
 * coords: The texture coordinates
 * array_index: The 0-based array index.
 * depth_ref: The reference value to compare the sampled depth value against
`
).
paramsSubcasesOnly((u) =>
u.
combine('S', ['clamp-to-edge', 'repeat', 'mirror-repeat']).
combine('C', ['i32', 'u32']).
combine('C_value', [-1, 0, 1, 2, 3, 4]).
combine('coords', generateCoordBoundaries(3)).
combine('depth_ref', [-1 /* smaller ref */, 0 /* equal ref */, 1 /* larger ref */])
).
unimplemented();

g.test('sampled_array_2d_coords').
specURL('https://www.w3.org/TR/WGSL/#texturegathercompare').
desc(
  `
fn textureGatherCompare(t: texture_depth_2d, s: sampler_comparison, coords: vec2<f32>, depth_ref: f32) -> vec4<f32>
fn textureGatherCompare(t: texture_depth_2d, s: sampler_comparison, coords: vec2<f32>, depth_ref: f32, offset: vec2<i32>) -> vec4<f32>

Parameters:
 * t: The depth texture to read from
 * s: The sampler_comparison
 * coords: The texture coordinates
 * depth_ref: The reference value to compare the sampled depth value against
 * offset:
    - The optional texel offset applied to the unnormalized texture coordinate before sampling the texture.
      This offset is applied before applying any texture wrapping modes.
    - The offset expression must be a creation-time expression (e.g. vec2<i32>(1, 2)).
    - Each offset component must be at least -8 and at most 7.
      Values outside of this range will result in a shader-creation error.
`
).
paramsSubcasesOnly((u) =>
u.
combine('S', ['clamp-to-edge', 'repeat', 'mirror-repeat']).
combine('coords', generateCoordBoundaries(2)).
combine('depth_ref', [-1 /* smaller ref */, 0 /* equal ref */, 1 /* larger ref */]).
combine('offset', generateOffsets(2))
).
unimplemented();

g.test('sampled_array_3d_coords').
specURL('https://www.w3.org/TR/WGSL/#texturegathercompare').
desc(
  `
fn textureGatherCompare(t: texture_depth_cube, s: sampler_comparison, coords: vec3<f32>, depth_ref: f32) -> vec4<f32>

Parameters:
 * t: The depth texture to read from
 * s: The sampler_comparison
 * coords: The texture coordinates
 * depth_ref: The reference value to compare the sampled depth value against
`
).
paramsSubcasesOnly((u) =>
u.
combine('S', ['clamp-to-edge', 'repeat', 'mirror-repeat']).
combine('coords', generateCoordBoundaries(3)).
combine('depth_ref', [-1 /* smaller ref */, 0 /* equal ref */, 1 /* larger ref */])
).
unimplemented();