/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Execution tests for the 'textureNumSamples' builtin function

Returns the number samples per texel in a multisampled texture.
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';

export const g = makeTestGroup(GPUTest);

g.test('sampled')
  .specURL('https://www.w3.org/TR/WGSL/#texturenumsamples')
  .desc(
    `
T, a sampled type.

fn textureNumSamples(t: texture_multisampled_2d<T>) -> u32

Parameters
 * t The multisampled texture.
`
  )
  .params(u => u.beginSubcases().combine('sampled_type', ['f32-only', 'i32', 'u32']))
  .unimplemented();

g.test('depth')
  .specURL('https://www.w3.org/TR/WGSL/#texturenumsamples')
  .desc(
    `
fn textureNumSamples(t: texture_depth_multisampled_2d) -> u32

Parameters
 * t The multisampled texture.
`
  )
  .unimplemented();
