/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution tests for the 'textureSampleBias' builtin function

Samples a texture with a bias to the mip level.
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';

import { generateCoordBoundaries, generateOffsets } from './utils.js';

export const g = makeTestGroup(GPUTest);

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
paramsSubcasesOnly((u) =>
u.
combine('S', ['clamp-to-edge', 'repeat', 'mirror-repeat']).
combine('coords', generateCoordBoundaries(2)).
combine('bias', [-16.1, -16, 0, 1, 15.99, 16]).
combine('offset', generateOffsets(2))
).
unimplemented();

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
combine('texture_type', ['texture_3d', 'texture_cube']).
beginSubcases().
combine('S', ['clamp-to-edge', 'repeat', 'mirror-repeat']).
combine('coords', generateCoordBoundaries(3)).
combine('bias', [-16.1, -16, 0, 1, 15.99, 16]).
combine('offset', generateOffsets(3))
).
unimplemented();

g.test('arrayed_2d_coords').
specURL('https://www.w3.org/TR/WGSL/#texturesamplebias').
desc(
  `
C: i32, u32

fn textureSampleBias(t: texture_2d_array<f32>, s: sampler, coords: vec2<f32>, array_index: C, bias: f32) -> vec4<f32>
fn textureSampleBias(t: texture_2d_array<f32>, s: sampler, coords: vec2<f32>, array_index: C, bias: f32, offset: vec2<i32>) -> vec4<f32>

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
paramsSubcasesOnly((u) =>
u.
combine('S', ['clamp-to-edge', 'repeat', 'mirror-repeat']).
combine('coords', generateCoordBoundaries(2)).
combine('C', ['i32', 'u32']).
combine('C_value', [-1, 0, 1, 2, 3, 4])
/* array_index not param'd as out-of-bounds is implementation specific */.
combine('bias', [-16.1, -16, 0, 1, 15.99, 16]).
combine('offset', generateOffsets(2))
).
unimplemented();

g.test('arrayed_3d_coords').
specURL('https://www.w3.org/TR/WGSL/#texturesamplebias').
desc(
  `
C: i32, u32

fn textureSampleBias(t: texture_cube_array<f32>, s: sampler, coords: vec3<f32>, array_index: C, bias: f32) -> vec4<f32>

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
paramsSubcasesOnly((u) =>
u.
combine('S', ['clamp-to-edge', 'repeat', 'mirror-repeat']).
combine('coords', generateCoordBoundaries(3)).
combine('C', ['i32', 'u32']).
combine('C_value', [-1, 0, 1, 2, 3, 4])
/* array_index not param'd as out-of-bounds is implementation specific */.
combine('bias', [-16.1, -16, 0, 1, 15.99, 16])
).
unimplemented();