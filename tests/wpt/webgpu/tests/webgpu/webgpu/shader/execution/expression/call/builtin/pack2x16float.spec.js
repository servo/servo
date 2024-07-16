/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Converts two floating point values to half-precision floating point numbers, and then combines them into one u32 value.
Component e[i] of the input is converted to a IEEE-754 binary16 value,
which is then placed in bits 16 × i through 16 × i + 15 of the result.
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';
import { Type } from '../../../../../util/conversion.js';
import { allInputSources, run } from '../../expression.js';

import { builtin } from './builtin.js';
import { d } from './pack2x16float.cache.js';

export const g = makeTestGroup(GPUTest);

g.test('pack').
specURL('https://www.w3.org/TR/WGSL/#pack-builtin-functions').
desc(
  `
@const fn pack2x16float(e: vec2<f32>) -> u32
`
).
params((u) => u.combine('inputSource', allInputSources)).
fn(async (t) => {
  const cases = await d.get(t.params.inputSource === 'const' ? 'f32_const' : 'f32_non_const');
  await run(t, builtin('pack2x16float'), [Type.vec2f], Type.u32, t.params, cases);
});