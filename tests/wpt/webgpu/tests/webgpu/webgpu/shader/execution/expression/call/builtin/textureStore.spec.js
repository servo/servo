/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Writes a single texel to a texture.

The channel format T depends on the storage texel format F.
See the texel format table for the mapping of texel format to channel format.

Note: An out-of-bounds access occurs if:
 * any element of coords is outside the range [0, textureDimensions(t)) for the corresponding element, or
 * array_index is outside the range of [0, textureNumLayers(t))

If an out-of-bounds access occurs, the built-in function may do any of the following:
 * not be executed
 * store value to some in bounds texel
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';
import { TexelFormats } from '../../../../types.js';

import { generateCoordBoundaries } from './utils.js';

export const g = makeTestGroup(GPUTest);

g.test('store_1d_coords')
  .specURL('https://www.w3.org/TR/WGSL/#texturestore')
  .desc(
    `
C is i32 or u32

fn textureStore(t: texture_storage_1d<F,write>, coords: C, value: vec4<T>)

Parameters:
 * t  The sampled, depth, or external texture to sample.
 * s  The sampler type.
 * coords The texture coordinates used for sampling.
 * value The new texel value
`
  )
  .params(u =>
    u
      .combineWithParams(TexelFormats)
      .beginSubcases()
      .combine('coords', generateCoordBoundaries(1))
      .combine('C', ['i32', 'u32'])
  )
  .unimplemented();

g.test('store_2d_coords')
  .specURL('https://www.w3.org/TR/WGSL/#texturestore')
  .desc(
    `
C is i32 or u32

fn textureStore(t: texture_storage_2d<F,write>, coords: vec2<C>, value: vec4<T>)

Parameters:
 * t  The sampled, depth, or external texture to sample.
 * s  The sampler type.
 * coords The texture coordinates used for sampling.
 * value The new texel value
`
  )
  .params(u =>
    u
      .combineWithParams(TexelFormats)
      .beginSubcases()
      .combine('coords', generateCoordBoundaries(2))
      .combine('C', ['i32', 'u32'])
  )
  .unimplemented();

g.test('store_array_2d_coords')
  .specURL('https://www.w3.org/TR/WGSL/#texturestore')
  .desc(
    `
C is i32 or u32

fn textureStore(t: texture_storage_2d_array<F,write>, coords: vec2<C>, array_index: C, value: vec4<T>)

Parameters:
 * t  The sampled, depth, or external texture to sample.
 * s  The sampler type.
 * array_index The 0-based texture array index
 * coords The texture coordinates used for sampling.
 * value The new texel value
`
  )
  .params(
    u =>
      u
        .combineWithParams(TexelFormats)
        .beginSubcases()
        .combine('coords', generateCoordBoundaries(2))
        .combine('C', ['i32', 'u32'])
        .combine('C_value', [-1, 0, 1, 2, 3, 4])
    /* array_index not param'd as out-of-bounds is implementation specific */
  )
  .unimplemented();

g.test('store_3d_coords')
  .specURL('https://www.w3.org/TR/WGSL/#texturestore')
  .desc(
    `
C is i32 or u32

fn textureStore(t: texture_storage_3d<F,write>, coords: vec3<C>, value: vec4<T>)

Parameters:
 * t  The sampled, depth, or external texture to sample.
 * s  The sampler type.
 * coords The texture coordinates used for sampling.
 * value The new texel value
`
  )
  .params(u =>
    u
      .combineWithParams(TexelFormats)
      .beginSubcases()
      .combine('coords', generateCoordBoundaries(3))
      .combine('C', ['i32', 'u32'])
  )
  .unimplemented();
