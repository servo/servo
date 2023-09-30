/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Execution tests for the 'inverseSqrt' builtin function

S is AbstractFloat, f32, f16
T is S or vecN<S>
@const fn inverseSqrt(e: T ) -> T
Returns the reciprocal of sqrt(e). Component-wise when T is a vector.
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';
import { kValue } from '../../../../../util/constants.js';
import { TypeF32, TypeF16 } from '../../../../../util/conversion.js';
import { FP } from '../../../../../util/floating_point.js';
import { biasedRange, linearRange } from '../../../../../util/math.js';
import { makeCaseCache } from '../../case_cache.js';
import { allInputSources, run } from '../../expression.js';

import { builtin } from './builtin.js';

export const g = makeTestGroup(GPUTest);

export const d = makeCaseCache('inverseSqrt', {
  f32: () => {
    return FP.f32.generateScalarToIntervalCases(
      [
        // 0 < x <= 1 linearly spread
        ...linearRange(kValue.f32.positive.min, 1, 100),
        // 1 <= x < 2^32, biased towards 1
        ...biasedRange(1, 2 ** 32, 1000),
      ],

      'unfiltered',
      FP.f32.inverseSqrtInterval
    );
  },
  f16: () => {
    return FP.f16.generateScalarToIntervalCases(
      [
        // 0 < x <= 1 linearly spread
        ...linearRange(kValue.f16.positive.min, 1, 100),
        // 1 <= x < 2^15, biased towards 1
        ...biasedRange(1, 2 ** 15, 1000),
      ],

      'unfiltered',
      FP.f16.inverseSqrtInterval
    );
  },
});

g.test('abstract_float')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(`abstract float tests`)
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .unimplemented();

g.test('f32')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(`f32 tests`)
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .fn(async t => {
    const cases = await d.get('f32');
    await run(t, builtin('inverseSqrt'), [TypeF32], TypeF32, t.params, cases);
  });

g.test('f16')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(`f16 tests`)
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .beforeAllSubcases(t => {
    t.selectDeviceOrSkipTestCase('shader-f16');
  })
  .fn(async t => {
    const cases = await d.get('f16');
    await run(t, builtin('inverseSqrt'), [TypeF16], TypeF16, t.params, cases);
  });
