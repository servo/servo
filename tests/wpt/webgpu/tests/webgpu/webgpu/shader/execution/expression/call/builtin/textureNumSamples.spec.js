/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution tests for the 'textureNumSamples' builtin function

Returns the number samples per texel in a multisampled texture.
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { AllFeaturesMaxLimitsGPUTest } from '../../../../../gpu_test.js';
import { kShaderStages } from '../../../../validation/decl/util.js';

import { executeTextureQueryAndExpectResult, kSampleTypeInfo } from './texture_utils.js';

export const g = makeTestGroup(AllFeaturesMaxLimitsGPUTest);

g.test('sampled').
specURL('https://www.w3.org/TR/WGSL/#texturenumsamples').
desc(
  `
T, a sampled type.

fn textureNumSamples(t: texture_multisampled_2d<T>) -> u32

Parameters
 * t The multisampled texture.
`
).
params((u) =>
u.
beginSubcases().
combine('stage', kShaderStages).
combine('sampled_type', ['f32', 'i32', 'u32'])
).
fn((t) => {
  const { stage, sampled_type } = t.params;
  const { format } = kSampleTypeInfo[sampled_type];

  t.skipIfTextureFormatNotMultisampled(format);

  const sampleCount = 4;
  const texture = t.createTextureTracked({
    format,
    usage: GPUTextureUsage.TEXTURE_BINDING | GPUTextureUsage.RENDER_ATTACHMENT,
    size: [1, 1, 1],
    sampleCount
  });

  const code = `
@group(0) @binding(0) var t: texture_multisampled_2d<${sampled_type}>;
@group(0) @binding(1) var<storage, read_write> result: u32;
fn getValue() -> u32 {
  return textureNumSamples(t);
}
    `;

  const expected = [sampleCount];
  executeTextureQueryAndExpectResult(t, stage, code, texture, {}, expected);
});

g.test('depth').
specURL('https://www.w3.org/TR/WGSL/#texturenumsamples').
desc(
  `
fn textureNumSamples(t: texture_depth_multisampled_2d) -> u32

Parameters
 * t The multisampled texture.
`
).
params((u) => u.beginSubcases().combine('stage', kShaderStages)).
fn((t) => {
  const { stage } = t.params;
  const sampleCount = 4;
  const texture = t.createTextureTracked({
    format: 'depth32float',
    usage: GPUTextureUsage.TEXTURE_BINDING | GPUTextureUsage.RENDER_ATTACHMENT,
    size: [1, 1, 1],
    sampleCount
  });

  const code = `
@group(0) @binding(0) var t: texture_depth_multisampled_2d;
@group(0) @binding(1) var<storage, read_write> result: u32;
fn getValue() -> u32 {
  return textureNumSamples(t);
}
    `;

  const expected = [sampleCount];
  executeTextureQueryAndExpectResult(t, stage, code, texture, {}, expected);
});