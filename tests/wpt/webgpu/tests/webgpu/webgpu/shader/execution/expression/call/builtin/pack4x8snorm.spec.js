/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Converts four normalized floating point values to 8-bit signed integers, and then combines them into one u32 value.
Component e[i] of the input is converted to an 8-bit twos complement integer value
⌊ 0.5 + 127 × min(1, max(-1, e[i])) ⌋ which is then placed in
bits 8 × i through 8 × i + 7 of the result.
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';
import { kValue } from '../../../../../util/constants.js';
import {
  f32,
  pack4x8snorm,
  TypeF32,
  TypeU32,
  TypeVec,
  u32,
  vec4,
} from '../../../../../util/conversion.js';
import { quantizeToF32, vectorF32Range } from '../../../../../util/math.js';
import { allInputSources, run } from '../../expression.js';

import { builtin } from './builtin.js';

export const g = makeTestGroup(GPUTest);

g.test('pack')
  .specURL('https://www.w3.org/TR/WGSL/#pack-builtin-functions')
  .desc(
    `
@const fn pack4x8snorm(e: vec4<f32>) -> u32
`
  )
  .params(u => u.combine('inputSource', allInputSources))
  .fn(async t => {
    const makeCase = vals => {
      const vals_f32 = new Array(4);
      for (const idx in vals) {
        vals[idx] = quantizeToF32(vals[idx]);
        vals_f32[idx] = f32(vals[idx]);
      }

      return { input: [vec4(...vals_f32)], expected: u32(pack4x8snorm(...vals)) };
    };

    // Returns a value normalized to [-1, 1].
    const normalizeF32 = n => {
      return n / kValue.f32.positive.max;
    };

    const cases = vectorF32Range(4).flatMap(v => {
      return [makeCase(v), makeCase(v.map(normalizeF32))];
    });

    await run(t, builtin('pack4x8snorm'), [TypeVec(4, TypeF32)], TypeU32, t.params, cases);
  });
