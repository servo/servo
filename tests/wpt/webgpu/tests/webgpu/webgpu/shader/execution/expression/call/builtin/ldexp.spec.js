/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Execution tests for the 'ldexp' builtin function

S is AbstractFloat, f32, f16
T is S or vecN<S>

K is AbstractInt, i32
I is K or vecN<K>, where
  I is a scalar if T is a scalar, or a vector when T is a vector

@const fn ldexp(e1: T ,e2: I ) -> T
Returns e1 * 2^e2. Component-wise when T is a vector.
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { assert } from '../../../../../../common/util/util.js';
import { GPUTest } from '../../../../../gpu_test.js';
import { anyOf } from '../../../../../util/compare.js';
import { i32, TypeF32, TypeF16, TypeI32 } from '../../../../../util/conversion.js';
import { FP } from '../../../../../util/floating_point.js';
import {
  biasedRange,
  fullF32Range,
  fullF16Range,
  fullI32Range,
  quantizeToI32,
} from '../../../../../util/math.js';
import { makeCaseCache } from '../../case_cache.js';
import { allInputSources, run } from '../../expression.js';

import { builtin } from './builtin.js';

export const g = makeTestGroup(GPUTest);

const bias = {
  f32: 127,
  f16: 15,
};

// ldexpInterval's return interval doesn't cover the flush-to-zero cases when e2 + bias <= 0, thus
// special examination is required.
// See the comment block on ldexpInterval for more details
// e2 is an integer (i32) while e1 is float.
const makeCase = (trait, e1, e2) => {
  const FPTrait = FP[trait];
  e1 = FPTrait.quantize(e1);
  // e2 should be in i32 range for the convinience.
  assert(-2147483648 <= e2 && e2 <= 2147483647, 'e2 should be in i32 range');
  e2 = quantizeToI32(e2);

  const expected = FPTrait.ldexpInterval(e1, e2);

  // Result may be zero if e2 + bias <= 0
  if (e2 + bias[trait] <= 0) {
    return {
      input: [FPTrait.scalarBuilder(e1), i32(e2)],
      expected: anyOf(expected, FPTrait.constants().zeroInterval),
    };
  }

  return { input: [FPTrait.scalarBuilder(e1), i32(e2)], expected };
};

export const d = makeCaseCache('ldexp', {
  f32_non_const: () => {
    return fullF32Range().flatMap(e1 => fullI32Range().map(e2 => makeCase('f32', e1, e2)));
  },
  f32_const: () => {
    return fullF32Range().flatMap(e1 =>
      biasedRange(-bias.f32 - 10, bias.f32 + 1, 10).flatMap(e2 =>
        FP.f32.isFinite(e1 * 2 ** quantizeToI32(e2)) ? makeCase('f32', e1, e2) : []
      )
    );
  },
  f16_non_const: () => {
    return fullF16Range().flatMap(e1 => fullI32Range().map(e2 => makeCase('f16', e1, e2)));
  },
  f16_const: () => {
    return fullF16Range().flatMap(e1 =>
      biasedRange(-bias.f16 - 10, bias.f16 + 1, 10).flatMap(e2 =>
        FP.f16.isFinite(e1 * 2 ** quantizeToI32(e2)) ? makeCase('f16', e1, e2) : []
      )
    );
  },
});

g.test('abstract_float')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(
    `
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .unimplemented();

g.test('f32')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(`f32 tests`)
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .fn(async t => {
    const cases = await d.get(t.params.inputSource === 'const' ? 'f32_const' : 'f32_non_const');
    await run(t, builtin('ldexp'), [TypeF32, TypeI32], TypeF32, t.params, cases);
  });

g.test('f16')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(`f16 tests`)
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .beforeAllSubcases(t => {
    t.selectDeviceOrSkipTestCase('shader-f16');
  })
  .fn(async t => {
    const cases = await d.get(t.params.inputSource === 'const' ? 'f16_const' : 'f16_non_const');
    await run(t, builtin('ldexp'), [TypeF16, TypeI32], TypeF16, t.params, cases);
  });
