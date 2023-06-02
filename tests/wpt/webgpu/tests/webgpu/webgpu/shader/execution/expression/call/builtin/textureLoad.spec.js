/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Execution tests for the 'textureLoad' builtin function

Reads a single texel from a texture without sampling or filtering.

Returns the unfiltered texel data.

An out of bounds access occurs if:
 * any element of coords is outside the range [0, textureDimensions(t, level)) for the corresponding element, or
 * array_index is outside the range [0, textureNumLayers(t)), or
 * level is outside the range [0, textureNumLevels(t))

If an out of bounds access occurs, the built-in function returns one of:
 * The data for some texel within bounds of the texture
 * A vector (0,0,0,0) or (0,0,0,1) of the appropriate type for non-depth textures
 * 0.0 for depth textures
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';

import { generateCoordBoundaries } from './utils.js';

export const g = makeTestGroup(GPUTest);

g.test('sampled_1d')
  .specURL('https://www.w3.org/TR/WGSL/#textureload')
  .desc(
    `
C is i32 or u32

fn textureLoad(t: texture_1d<T>, coords: C, level: C) -> vec4<T>

Parameters:
 * t: The sampled texture to read from
 * coords: The 0-based texel coordinate
 * level: The mip level, with level 0 containing a full size version of the texture
`
  )
  .params(u =>
    u
      .combine('C', ['i32', 'u32'])
      .combine('coords', generateCoordBoundaries(1))
      .combine('level', [-1, 0, `numlevels-1`, `numlevels`])
  )
  .unimplemented();

g.test('sampled_2d')
  .specURL('https://www.w3.org/TR/WGSL/#textureload')
  .desc(
    `
C is i32 or u32

fn textureLoad(t: texture_2d<T>, coords: vec2<C>, level: C) -> vec4<T>

Parameters:
 * t: The sampled texture to read from
 * coords: The 0-based texel coordinate
 * level: The mip level, with level 0 containing a full size version of the texture
`
  )
  .params(u =>
    u
      .combine('C', ['i32', 'u32'])
      .combine('coords', generateCoordBoundaries(2))
      .combine('level', [-1, 0, `numlevels-1`, `numlevels`])
  )
  .unimplemented();

g.test('sampled_3d')
  .specURL('https://www.w3.org/TR/WGSL/#textureload')
  .desc(
    `
C is i32 or u32

fn textureLoad(t: texture_3d<T>, coords: vec3<C>, level: C) -> vec4<T>

Parameters:
 * t: The sampled texture to read from
 * coords: The 0-based texel coordinate
 * level: The mip level, with level 0 containing a full size version of the texture
`
  )
  .params(u =>
    u
      .combine('C', ['i32', 'u32'])
      .combine('coords', generateCoordBoundaries(3))
      .combine('level', [-1, 0, `numlevels-1`, `numlevels`])
  )
  .unimplemented();

g.test('multisampled')
  .specURL('https://www.w3.org/TR/WGSL/#textureload')
  .desc(
    `
C is i32 or u32

fn textureLoad(t: texture_multisampled_2d<T>, coords: vec2<C>, sample_index: C)-> vec4<T>
fn textureLoad(t: texture_depth_multisampled_2d, coords: vec2<C>, sample_index: C)-> f32

Parameters:
 * t: The sampled texture to read from
 * coords: The 0-based texel coordinate
 * sample_index: The 0-based sample index of the multisampled texture
`
  )
  .params(u =>
    u
      .combine('texture_type', ['texture_multisampled_2d', 'texture_depth_multisampled_2d'])
      .beginSubcases()
      .combine('C', ['i32', 'u32'])
      .combine('coords', generateCoordBoundaries(2))
      .combine('sample_index', [-1, 0, `sampleCount-1`, `sampleCount`])
  )
  .unimplemented();

g.test('depth')
  .specURL('https://www.w3.org/TR/WGSL/#textureload')
  .desc(
    `
C is i32 or u32

fn textureLoad(t: texture_depth_2d, coords: vec2<C>, level: C) -> f32

Parameters:
 * t: The sampled texture to read from
 * coords: The 0-based texel coordinate
 * level: The mip level, with level 0 containing a full size version of the texture
`
  )
  .paramsSubcasesOnly(u =>
    u
      .combine('C', ['i32', 'u32'])
      .combine('coords', generateCoordBoundaries(2))
      .combine('level', [-1, 0, `numlevels-1`, `numlevels`])
  )
  .unimplemented();

g.test('external')
  .specURL('https://www.w3.org/TR/WGSL/#textureload')
  .desc(
    `
C is i32 or u32

fn textureLoad(t: texture_external, coords: vec2<C>) -> vec4<f32>

Parameters:
 * t: The sampled texture to read from
 * coords: The 0-based texel coordinate
`
  )
  .paramsSubcasesOnly(u =>
    u.combine('C', ['i32', 'u32']).combine('coords', generateCoordBoundaries(2))
  )
  .unimplemented();

g.test('arrayed')
  .specURL('https://www.w3.org/TR/WGSL/#textureload')
  .desc(
    `
C is i32 or u32

fn textureLoad(t: texture_2d_array<T>, coords: vec2<C>, array_index: C, level: C) -> vec4<T>
fn textureLoad(t: texture_depth_2d_array, coords: vec2<C>, array_index: C, level: C) -> f32

Parameters:
 * t: The sampled texture to read from
 * coords: The 0-based texel coordinate
 * array_index: The 0-based texture array index
 * level: The mip level, with level 0 containing a full size version of the texture
`
  )
  .params(u =>
    u
      .combine('texture_type', ['texture_2d_array', 'texture_depth_2d_array'])
      .beginSubcases()
      .combine('C', ['i32', 'u32'])
      .combine('coords', generateCoordBoundaries(2))
      .combine('array_index', [-1, 0, `numlayers-1`, `numlayers`])
      .combine('level', [-1, 0, `numlevels-1`, `numlevels`])
  )
  .unimplemented();
