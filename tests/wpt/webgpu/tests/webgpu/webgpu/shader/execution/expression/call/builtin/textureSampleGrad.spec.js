/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Samples a texture using explicit gradients.
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';

import { generateCoordBoundaries, generateOffsets } from './utils.js';

export const g = makeTestGroup(GPUTest);

g.test('sampled_2d_coords').
specURL('https://www.w3.org/TR/WGSL/#texturesamplegrad').
desc(
  `
fn textureSampleGrad(t: texture_2d<f32>, s: sampler, coords: vec2<f32>, ddx: vec2<f32>, ddy: vec2<f32>) -> vec4<f32>
fn textureSampleGrad(t: texture_2d<f32>, s: sampler, coords: vec2<f32>, ddx: vec2<f32>, ddy: vec2<f32>, offset: vec2<i32>) -> vec4<f32>

Parameters:
 * t  The sampled texture.
 * s  The sampler type.
 * coords The texture coordinates used for sampling.
 * ddx The x direction derivative vector used to compute the sampling locations
 * ddy The y direction derivative vector used to compute the sampling locations
 * offset
    * The optional texel offset applied to the unnormalized texture coordinate before sampling the texture.
    * This offset is applied before applying any texture wrapping modes.
    * The offset expression must be a creation-time expression (e.g. vec2<i32>(1, 2)).
    * Each offset component must be at least -8 and at most 7.
      Values outside of this range will result in a shader-creation error.
`
).
paramsSubcasesOnly((u) =>
u.
combine('S', ['clamp-to-edge', 'repeat', 'mirror-repeat']).
combine('coords', generateCoordBoundaries(2)).
combine('offset', generateOffsets(2))
).
unimplemented();

g.test('sampled_3d_coords').
specURL('https://www.w3.org/TR/WGSL/#texturesamplegrad').
desc(
  `
fn textureSampleGrad(t: texture_3d<f32>, s: sampler, coords: vec3<f32>, ddx: vec3<f32>, ddy: vec3<f32>) -> vec4<f32>
fn textureSampleGrad(t: texture_3d<f32>, s: sampler, coords: vec3<f32>, ddx: vec3<f32>, ddy: vec3<f32>, offset: vec3<i32>) -> vec4<f32>
fn textureSampleGrad(t: texture_cube<f32>, s: sampler, coords: vec3<f32>, ddx: vec3<f32>, ddy: vec3<f32>) -> vec4<f32>

Parameters:
 * t  The sampled texture.
 * s  The sampler type.
 * ddx The x direction derivative vector used to compute the sampling locations
 * ddy The y direction derivative vector used to compute the sampling locations
 * coords The texture coordinates used for sampling.
 * offset
    * The optional texel offset applied to the unnormalized texture coordinate before sampling the texture.
    * This offset is applied before applying any texture wrapping modes.
    * The offset expression must be a creation-time expression (e.g. vec2<i32>(1, 2)).
    * Each offset component must be at least -8 and at most 7.
      Values outside of this range will result in a shader-creation error.
`
).
paramsSubcasesOnly((u) =>
u.
combine('S', ['clamp-to-edge', 'repeat', 'mirror-repeat']).
combine('coords', generateCoordBoundaries(3)).
combine('offset', generateOffsets(3))
).
unimplemented();

g.test('sampled_array_2d_coords').
specURL('https://www.w3.org/TR/WGSL/#texturesamplegrad').
desc(
  `
C is i32 or u32

fn textureSampleGrad(t: texture_2d_array<f32>, s: sampler, coords: vec2<f32>, array_index: C, ddx: vec2<f32>, ddy: vec2<f32>) -> vec4<f32>
fn textureSampleGrad(t: texture_2d_array<f32>, s: sampler, coords: vec2<f32>, array_index: C, ddx: vec2<f32>, ddy: vec2<f32>, offset: vec2<i32>) -> vec4<f32>

Parameters:
 * t  The sampled texture.
 * s  The sampler type.
 * coords The texture coordinates used for sampling.
 * array_index The 0-based texture array index to sample.
 * ddx The x direction derivative vector used to compute the sampling locations
 * ddy The y direction derivative vector used to compute the sampling locations
 * offset
    * The optional texel offset applied to the unnormalized texture coordinate before sampling the texture.
    * This offset is applied before applying any texture wrapping modes.
    * The offset expression must be a creation-time expression (e.g. vec2<i32>(1, 2)).
    * Each offset component must be at least -8 and at most 7.
      Values outside of this range will result in a shader-creation error.
`
).
paramsSubcasesOnly((u) =>
u.
combine('S', ['clamp-to-edge', 'repeat', 'mirror-repeat']).
combine('C', ['i32', 'u32']).
combine('C_value', [-1, 0, 1, 2, 3, 4]).
combine('coords', generateCoordBoundaries(2))
/* array_index not param'd as out-of-bounds is implementation specific */.
combine('offset', generateOffsets(2))
).
unimplemented();

g.test('sampled_array_3d_coords').
specURL('https://www.w3.org/TR/WGSL/#texturesamplegrad').
desc(
  `
C is i32 or u32

fn textureSampleGrad(t: texture_cube_array<f32>, s: sampler, coords: vec3<f32>, array_index: C, ddx: vec3<f32>, ddy: vec3<f32>) -> vec4<f32>

Parameters:
 * t  The sampled texture.
 * s  The sampler type.
 * coords The texture coordinates used for sampling.
 * array_index The 0-based texture array index to sample.
 * ddx The x direction derivative vector used to compute the sampling locations
 * ddy The y direction derivative vector used to compute the sampling locations
 * offset
    * The optional texel offset applied to the unnormalized texture coordinate before sampling the texture.
    * This offset is applied before applying any texture wrapping modes.
    * The offset expression must be a creation-time expression (e.g. vec2<i32>(1, 2)).
    * Each offset component must be at least -8 and at most 7.
      Values outside of this range will result in a shader-creation error.
`
).
paramsSubcasesOnly((u) =>
u.
combine('S', ['clamp-to-edge', 'repeat', 'mirror-repeat']).
combine('C', ['i32', 'u32']).
combine('C_value', [-1, 0, 1, 2, 3, 4]).
combine('coords', generateCoordBoundaries(3))
).
unimplemented();