/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Decomposes a 32-bit value into two 16-bit chunks, and reinterpets each chunk as
a floating point value.
Component i of the result is the f32 representation of v, where v is the
interpretation of bits 16×i through 16×i+15 of e as an IEEE-754 binary16 value.
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';
import { Type } from '../../../../../util/conversion.js';
import { allInputSources, run } from '../../expression.js';

import { builtin } from './builtin.js';
import { d } from './unpack2x16float.cache.js';

export const g = makeTestGroup(GPUTest);

g.test('unpack').
specURL('https://www.w3.org/TR/WGSL/#unpack-builtin-functions').
desc(
  `
@const fn unpack2x16float(e: u32) -> vec2<f32>
`
).
params((u) => u.combine('inputSource', allInputSources)).
fn(async (t) => {
  const cases = await d.get(t.params.inputSource === 'const' ? 'u32_const' : 'u32_non_const');
  await run(t, builtin('unpack2x16float'), [Type.u32], Type.vec2f, t.params, cases);
});