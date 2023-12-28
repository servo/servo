/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Samples a depth texture and compares the sampled depth values against a reference value.

The textureSampleCompareLevel function is the same as textureSampleCompare, except that:

 * textureSampleCompareLevel always samples texels from mip level 0.
   * The function does not compute derivatives.
   * There is no requirement for textureSampleCompareLevel to be invoked in uniform control flow.
 * textureSampleCompareLevel may be invoked in any shader stage.
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';

import { generateCoordBoundaries, generateOffsets } from './utils.js';

export const g = makeTestGroup(GPUTest);

g.test('stage').
specURL('https://www.w3.org/TR/WGSL/#texturesamplecomparelevel').
desc(
  `
Tests that 'textureSampleCompareLevel' maybe called in any shader stage.
`
).
params((u) => u.combine('stage', ['fragment', 'vertex', 'compute'])).
unimplemented();

g.test('control_flow').
specURL('https://www.w3.org/TR/WGSL/#texturesamplecomparelevel').
desc(
  `
Tests that 'textureSampleCompareLevel' maybe called in non-uniform control flow.
`
).
params((u) => u.combine('stage', ['fragment', 'vertex', 'compute'])).
unimplemented();

g.test('2d_coords').
specURL('https://www.w3.org/TR/WGSL/#texturesamplecomparelevel').
desc(
  `
fn textureSampleCompareLevel(t: texture_depth_2d, s: sampler_comparison, coords: vec2<f32>, depth_ref: f32) -> f32
fn textureSampleCompareLevel(t: texture_depth_2d, s: sampler_comparison, coords: vec2<f32>, depth_ref: f32, offset: vec2<i32>) -> f32

Parameters:
 * t  The depth texture to sample.
 * s  The sampler_comparision type.
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
paramsSubcasesOnly((u) =>
u.
combine('S', ['clamp-to-edge', 'repeat', 'mirror-repeat']).
combine('coords', generateCoordBoundaries(2)).
combine('depth_ref', [-1 /* smaller ref */, 0 /* equal ref */, 1 /* larger ref */]).
combine('offset', generateOffsets(2))
).
unimplemented();

g.test('3d_coords').
specURL('https://www.w3.org/TR/WGSL/#texturesamplecomparelevel').
desc(
  `
fn textureSampleCompareLevel(t: texture_depth_cube, s: sampler_comparison, coords: vec3<f32>, depth_ref: f32) -> f32

Parameters:
 * t  The depth texture to sample.
 * s  The sampler_comparision type.
 * coords The texture coordinates used for sampling.
 * depth_ref The reference value to compare the sampled depth value against.
`
).
paramsSubcasesOnly((u) =>
u.
combine('S', ['clamp-to-edge', 'repeat', 'mirror-repeat']).
combine('coords', generateCoordBoundaries(3)).
combine('depth_ref', [-1 /* smaller ref */, 0 /* equal ref */, 1 /* larger ref */])
).
unimplemented();

g.test('arrayed_2d_coords').
specURL('https://www.w3.org/TR/WGSL/#texturesamplecomparelevel').
desc(
  `
C is i32 or u32

fn textureSampleCompareLevel(t: texture_depth_2d_array, s: sampler_comparison, coords: vec2<f32>, array_index: C, depth_ref: f32) -> f32
fn textureSampleCompareLevel(t: texture_depth_2d_array, s: sampler_comparison, coords: vec2<f32>, array_index: C, depth_ref: f32, offset: vec2<i32>) -> f32

Parameters:
 * t  The depth texture to sample.
 * s  The sampler_comparision type.
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
paramsSubcasesOnly((u) =>
u.
combine('S', ['clamp-to-edge', 'repeat', 'mirror-repeat']).
combine('coords', generateCoordBoundaries(2)).
combine('C', ['i32', 'u32']).
combine('C_value', [-1, 0, 1, 2, 3, 4])
/* array_index not param'd as out-of-bounds is implementation specific */.
combine('depth_ref', [-1 /* smaller ref */, 0 /* equal ref */, 1 /* larger ref */]).
combine('offset', generateOffsets(2))
).
unimplemented();

g.test('arrayed_3d_coords').
specURL('https://www.w3.org/TR/WGSL/#texturesamplecomparelevel').
desc(
  `
C is i32 or u32

fn textureSampleCompareLevel(t: texture_depth_cube_array, s: sampler_comparison, coords: vec3<f32>, array_index: C, depth_ref: f32) -> f32

Parameters:
 * t  The depth texture to sample.
 * s  The sampler_comparision type.
 * coords The texture coordinates used for sampling.
 * array_index: The 0-based texture array index to sample.
 * depth_ref The reference value to compare the sampled depth value against.
`
).
paramsSubcasesOnly((u) =>
u.
combine('S', ['clamp-to-edge', 'repeat', 'mirror-repeat']).
combine('coords', generateCoordBoundaries(3)).
combine('C', ['i32', 'u32']).
combine('C_value', [-1, 0, 1, 2, 3, 4])
/* array_index not param'd as out-of-bounds is implementation specific */.
combine('depth_ref', [-1 /* smaller ref */, 0 /* equal ref */, 1 /* larger ref */])
).
unimplemented();