/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Execution tests for the 'textureNumLayers' builtin function

Returns the number of layers (elements) of an array texture.
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';

export const g = makeTestGroup(GPUTest);

g.test('sampled')
  .specURL('https://www.w3.org/TR/WGSL/#texturenumlayers')
  .desc(
    `
T, a sampled type.

fn textureNumLayers(t: texture_2d_array<T>) -> u32
fn textureNumLayers(t: texture_cube_array<T>) -> u32

Parameters
 * t The sampled array texture.
`
  )
  .params(u =>
    u
      .combine('texture_type', ['texture_2d_array', 'texture_cube_array'])
      .beginSubcases()
      .combine('sampled_type', ['f32-only', 'i32', 'u32'])
  )
  .unimplemented();

g.test('arrayed')
  .specURL('https://www.w3.org/TR/WGSL/#texturenumlayers')
  .desc(
    `
fn textureNumLayers(t: texture_depth_2d_array) -> u32
fn textureNumLayers(t: texture_depth_cube_array) -> u32

Parameters
 * t The depth array texture.
`
  )
  .params(u => u.combine('texture_type', ['texture_depth_2d_array', 'texture_depth_cube_array']))
  .unimplemented();

g.test('storage')
  .specURL('https://www.w3.org/TR/WGSL/#texturenumlayers')
  .desc(
    `
F: rgba8unorm
   rgba8snorm
   rgba8uint
   rgba8sint
   rgba16uint
   rgba16sint
   rgba16float
   r32uint
   r32sint
   r32float
   rg32uint
   rg32sint
   rg32float
   rgba32uint
   rgba32sint
   rgba32float
A: read, write, read_write

fn textureNumLayers(t: texture_storage_2d_array<F,A>) -> u32

Parameters
 * t The sampled storage array texture.
`
  )
  .params(u =>
    u
      .beginSubcases()
      .combine('texel_format', [
        'rgba8unorm',
        'rgba8snorm',
        'rgba8uint',
        'rgba8sint',
        'rgba16uint',
        'rgba16sint',
        'rgba16float',
        'r32uint',
        'r32sint',
        'r32float',
        'rg32uint',
        'rg32sint',
        'rg32float',
        'rgba32uint',
        'rgba32sint',
        'rgba32float',
      ])
      .combine('access_mode', ['read', 'write', 'read_write'])
  )
  .unimplemented();
