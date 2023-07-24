/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Execution tests for the 'textureNumLevels' builtin function

Returns the number of mip levels of a texture.
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';

export const g = makeTestGroup(GPUTest);

g.test('sampled')
  .specURL('https://www.w3.org/TR/WGSL/#texturenumlevels')
  .desc(
    `
T, a sampled type.

fn textureNumLevels(t: texture_1d<T>) -> u32
fn textureNumLevels(t: texture_2d<T>) -> u32
fn textureNumLevels(t: texture_2d_array<T>) -> u32
fn textureNumLevels(t: texture_3d<T>) -> u32
fn textureNumLevels(t: texture_cube<T>) -> u32
fn textureNumLevels(t: texture_cube_array<T>) -> u32

Parameters
 * t The sampled array texture.
`
  )
  .params(u =>
    u
      .combine('texture_type', [
        'texture_1d',
        'texture_2d',
        'texture_2d_array',
        'texture_3d',
        'texture_cube',
        'texture_cube_array`',
      ])
      .beginSubcases()
      .combine('sampled_type', ['f32-only', 'i32', 'u32'])
  )
  .unimplemented();

g.test('depth')
  .specURL('https://www.w3.org/TR/WGSL/#texturenumlevels')
  .desc(
    `
fn textureNumLevels(t: texture_depth_2d) -> u32
fn textureNumLevels(t: texture_depth_2d_array) -> u32
fn textureNumLevels(t: texture_depth_cube) -> u32
fn textureNumLevels(t: texture_depth_cube_array) -> u32

Parameters
 * t The depth array texture.
`
  )
  .params(u =>
    u.combine('texture_type', [
      'texture_depth_2d',
      'texture_depth_2d_array',
      'texture_depth_cube',
      'texture_depth_cube_array',
    ])
  )
  .unimplemented();
