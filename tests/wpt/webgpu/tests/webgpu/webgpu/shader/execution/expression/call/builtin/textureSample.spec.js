/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Samples a texture.

Must only be used in a fragment shader stage.
Must only be invoked in uniform control flow.
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';

import { generateCoordBoundaries, generateOffsets } from './utils.js';

export const g = makeTestGroup(GPUTest);

g.test('stage')
  .specURL('https://www.w3.org/TR/WGSL/#texturesample')
  .desc(
    `
Tests that 'textureSample' can only be called in 'fragment' shaders.
`
  )
  .params(u => u.combine('stage', ['fragment', 'vertex', 'compute']))
  .unimplemented();

g.test('control_flow')
  .specURL('https://www.w3.org/TR/WGSL/#texturesample')
  .desc(
    `
Tests that 'textureSample' can only be called in uniform control flow.
`
  )
  .params(u => u.combine('stage', ['fragment', 'vertex', 'compute']))
  .unimplemented();

g.test('sampled_1d_coords')
  .specURL('https://www.w3.org/TR/WGSL/#texturesample')
  .desc(
    `
fn textureSample(t: texture_1d<f32>, s: sampler, coords: f32) -> vec4<f32>

Parameters:
 * t  The sampled, depth, or external texture to sample.
 * s  The sampler type.
 * coords The texture coordinates used for sampling.
`
  )
  .paramsSubcasesOnly(u =>
    u
      .combine('S', ['clamp-to-edge', 'repeat', 'mirror-repeat'])
      .combine('coords', generateCoordBoundaries(1))
  )
  .unimplemented();

g.test('sampled_2d_coords')
  .specURL('https://www.w3.org/TR/WGSL/#texturesample')
  .desc(
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
  )
  .paramsSubcasesOnly(u =>
    u
      .combine('S', ['clamp-to-edge', 'repeat', 'mirror-repeat'])
      .combine('coords', generateCoordBoundaries(2))
      .combine('offset', generateOffsets(2))
  )
  .unimplemented();

g.test('sampled_3d_coords')
  .specURL('https://www.w3.org/TR/WGSL/#texturesample')
  .desc(
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
`
  )
  .params(u =>
    u
      .combine('texture_type', ['texture_3d', 'texture_cube'])
      .beginSubcases()
      .combine('S', ['clamp-to-edge', 'repeat', 'mirror-repeat'])
      .combine('coords', generateCoordBoundaries(3))
      .combine('offset', generateOffsets(3))
  )
  .unimplemented();

g.test('depth_2d_coords')
  .specURL('https://www.w3.org/TR/WGSL/#texturesample')
  .desc(
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
  )
  .paramsSubcasesOnly(u =>
    u
      .combine('S', ['clamp-to-edge', 'repeat', 'mirror-repeat'])
      .combine('coords', generateCoordBoundaries(2))
      .combine('offset', generateOffsets(2))
  )
  .unimplemented();

g.test('sampled_array_2d_coords')
  .specURL('https://www.w3.org/TR/WGSL/#texturesample')
  .desc(
    `
C is i32 or u32

fn textureSample(t: texture_2d_array<f32>, s: sampler, coords: vec2<f32>, array_index: C) -> vec4<f32>
fn textureSample(t: texture_2d_array<f32>, s: sampler, coords: vec2<f32>, array_index: C, offset: vec2<i32>) -> vec4<f32>

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
  )
  .paramsSubcasesOnly(u =>
    u
      .combine('C', ['i32', 'u32'])
      .combine('C_value', [-1, 0, 1, 2, 3, 4])
      .combine('S', ['clamp-to-edge', 'repeat', 'mirror-repeat'])
      .combine('coords', generateCoordBoundaries(2))
      /* array_index not param'd as out-of-bounds is implementation specific */ .combine(
        'offset',
        generateOffsets(2)
      )
  )
  .unimplemented();

g.test('sampled_array_3d_coords')
  .specURL('https://www.w3.org/TR/WGSL/#texturesample')
  .desc(
    `
C is i32 or u32

fn textureSample(t: texture_cube_array<f32>, s: sampler, coords: vec3<f32>, array_index: C) -> vec4<f32>

Parameters:
 * t  The sampled, depth, or external texture to sample.
 * s  The sampler type.
 * coords The texture coordinates used for sampling.
 * array_index The 0-based texture array index to sample.
`
  )
  .paramsSubcasesOnly(
    u =>
      u
        .combine('C', ['i32', 'u32'])
        .combine('C_value', [-1, 0, 1, 2, 3, 4])
        .combine('S', ['clamp-to-edge', 'repeat', 'mirror-repeat'])
        .combine('coords', generateCoordBoundaries(3))
    /* array_index not param'd as out-of-bounds is implementation specific */
  )
  .unimplemented();

g.test('depth_3d_coords')
  .specURL('https://www.w3.org/TR/WGSL/#texturesample')
  .desc(
    `
fn textureSample(t: texture_depth_cube, s: sampler, coords: vec3<f32>) -> f32

Parameters:
 * t  The sampled, depth, or external texture to sample.
 * s  The sampler type.
 * coords The texture coordinates used for sampling.
`
  )
  .paramsSubcasesOnly(u =>
    u
      .combine('S', ['clamp-to-edge', 'repeat', 'mirror-repeat'])
      .combine('coords', generateCoordBoundaries(3))
  )
  .unimplemented();

g.test('depth_array_2d_coords')
  .specURL('https://www.w3.org/TR/WGSL/#texturesample')
  .desc(
    `
C is i32 or u32

fn textureSample(t: texture_depth_2d_array, s: sampler, coords: vec2<f32>, array_index: C) -> f32
fn textureSample(t: texture_depth_2d_array, s: sampler, coords: vec2<f32>, array_index: C, offset: vec2<i32>) -> f32

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
  )
  .paramsSubcasesOnly(u =>
    u
      .combine('C', ['i32', 'u32'])
      .combine('C_value', [-1, 0, 1, 2, 3, 4])
      .combine('S', ['clamp-to-edge', 'repeat', 'mirror-repeat'])
      .combine('coords', generateCoordBoundaries(2))
      /* array_index not param'd as out-of-bounds is implementation specific */ .combine(
        'offset',
        generateOffsets(2)
      )
  )
  .unimplemented();

g.test('depth_array_3d_coords')
  .specURL('https://www.w3.org/TR/WGSL/#texturesample')
  .desc(
    `
C is i32 or u32

fn textureSample(t: texture_depth_cube_array, s: sampler, coords: vec3<f32>, array_index: C) -> f32

Parameters:
 * t  The sampled, depth, or external texture to sample.
 * s  The sampler type.
 * coords The texture coordinates used for sampling.
 * array_index The 0-based texture array index to sample.
`
  )
  .paramsSubcasesOnly(
    u =>
      u
        .combine('C', ['i32', 'u32'])
        .combine('C_value', [-1, 0, 1, 2, 3, 4])
        .combine('S', ['clamp-to-edge', 'repeat', 'mirror-repeat'])
        .combine('coords', generateCoordBoundaries(3))
    /* array_index not param'd as out-of-bounds is implementation specific */
  )
  .unimplemented();
