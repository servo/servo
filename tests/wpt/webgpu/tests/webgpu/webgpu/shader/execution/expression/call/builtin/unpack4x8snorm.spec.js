/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Decomposes a 32-bit value into four 8-bit chunks, then reinterprets each chunk
as a signed normalized floating point value.
Component i of the result is max(v ÷ 127, -1), where v is the interpretation of
bits 8×i through 8×i+7 of e as a twos-complement signed integer.
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';
import { Type } from '../../../../../util/conversion.js';
import { allInputSources, run } from '../../expression.js';

import { builtin } from './builtin.js';
import { d } from './unpack4x8snorm.cache.js';

export const g = makeTestGroup(GPUTest);

g.test('unpack').
specURL('https://www.w3.org/TR/WGSL/#unpack-builtin-functions').
desc(
  `
@const fn unpack4x8snorm(e: u32) -> vec4<f32>
`
).
params((u) => u.combine('inputSource', allInputSources)).
fn(async (t) => {
  const cases = await d.get(t.params.inputSource === 'const' ? 'u32_const' : 'u32_non_const');
  await run(t, builtin('unpack4x8snorm'), [Type.u32], Type.vec4f, t.params, cases);
});