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
import { GPUTest } from '../../../../../gpu_test.js';
import { i32, TypeF32, TypeI32 } from '../../../../../util/conversion.js';
import { FP } from '../../../../../util/floating_point.js';
import {
  biasedRange,
  fullF32Range,
  fullI32Range,
  quantizeToI32,
} from '../../../../../util/math.js';
import { makeCaseCache } from '../../case_cache.js';
import { allInputSources, run } from '../../expression.js';

import { builtin } from './builtin.js';

export const g = makeTestGroup(GPUTest);

function makeCaseF32(e1, e2) {
  // Due to the heterogeneous types of the params to ldexp (f32 & i32),
  // makeBinaryToF32IntervalCase cannot be used here.
  e1 = FP.f32.quantize(e1);
  e2 = quantizeToI32(e2);
  const expected = FP.f32.ldexpInterval(e1, e2);
  return { input: [FP.f32.scalarBuilder(e1), i32(e2)], expected };
}

export const d = makeCaseCache('ldexp', {
  f32_non_const: () => {
    const cases = [];
    fullF32Range().forEach(e1 => {
      fullI32Range().forEach(e2 => {
        cases.push(makeCaseF32(e1, e2));
      });
    });
    return cases;
  },
  f32_const: () => {
    const cases = [];
    fullF32Range().forEach(e1 => {
      biasedRange(-128, 128, 10).forEach(e2 => {
        if (FP.f32.isFinite(e1 * Math.pow(2, e2))) {
          cases.push(makeCaseF32(e1, e2));
        }
      });
    });
    return cases;
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
  .unimplemented();
