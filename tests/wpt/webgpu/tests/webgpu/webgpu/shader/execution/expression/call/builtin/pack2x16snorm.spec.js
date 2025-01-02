/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Converts two normalized floating point values to 16-bit signed integers, and then combines them into one u32 value.
Component e[i] of the input is converted to a 16-bit twos complement integer value
⌊ 0.5 + 32767 × min(1, max(-1, e[i])) ⌋ which is then placed in
bits 16 × i through 16 × i + 15 of the result.
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';
import { kValue } from '../../../../../util/constants.js';
import { f32, pack2x16snorm, u32, vec2, Type } from '../../../../../util/conversion.js';
import { quantizeToF32, vectorF32Range } from '../../../../../util/math.js';

import { allInputSources, run } from '../../expression.js';

import { builtin } from './builtin.js';

export const g = makeTestGroup(GPUTest);

g.test('pack').
specURL('https://www.w3.org/TR/WGSL/#pack-builtin-functions').
desc(
  `
@const fn pack2x16snorm(e: vec2<f32>) -> u32
`
).
params((u) => u.combine('inputSource', allInputSources)).
fn(async (t) => {
  const makeCase = (x, y) => {
    x = quantizeToF32(x);
    y = quantizeToF32(y);
    return { input: [vec2(f32(x), f32(y))], expected: u32(pack2x16snorm(x, y)) };
  };

  // Returns a value normalized to [-1, 1].
  const normalizeF32 = (n) => {
    return n / kValue.f32.positive.max;
  };

  const cases = vectorF32Range(2).flatMap((v) => {
    return [
    makeCase(...v),
    makeCase(...v.map(normalizeF32))];

  });

  await run(t, builtin('pack2x16snorm'), [Type.vec2f], Type.u32, t.params, cases);
});