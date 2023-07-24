/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Execution tests for the 'quantizeToF16' builtin function

T is f32 or vecN<f32>
@const fn quantizeToF16(e: T ) -> T
Quantizes a 32-bit floating point value e as if e were converted to a IEEE 754
binary16 value, and then converted back to a IEEE 754 binary32 value.
Component-wise when T is a vector.
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';
import { kValue } from '../../../../../util/constants.js';
import { TypeF32 } from '../../../../../util/conversion.js';
import { FP } from '../../../../../util/floating_point.js';
import { fullF16Range, fullF32Range } from '../../../../../util/math.js';
import { makeCaseCache } from '../../case_cache.js';
import { allInputSources, run } from '../../expression.js';

import { builtin } from './builtin.js';

export const g = makeTestGroup(GPUTest);

export const d = makeCaseCache('quantizeToF16', {
  f32_const: () => {
    return FP.f32.generateScalarToIntervalCases(
      [
        kValue.f16.negative.min,
        kValue.f16.negative.max,
        kValue.f16.subnormal.negative.min,
        kValue.f16.subnormal.negative.max,
        kValue.f16.subnormal.positive.min,
        kValue.f16.subnormal.positive.max,
        kValue.f16.positive.min,
        kValue.f16.positive.max,
        ...fullF16Range(),
      ],

      'finite',
      FP.f32.quantizeToF16Interval
    );
  },
  f32_non_const: () => {
    return FP.f32.generateScalarToIntervalCases(
      [
        kValue.f16.negative.min,
        kValue.f16.negative.max,
        kValue.f16.subnormal.negative.min,
        kValue.f16.subnormal.negative.max,
        kValue.f16.subnormal.positive.min,
        kValue.f16.subnormal.positive.max,
        kValue.f16.positive.min,
        kValue.f16.positive.max,
        ...fullF32Range(),
      ],

      'unfiltered',
      FP.f32.quantizeToF16Interval
    );
  },
});

g.test('f32')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(`f32 tests`)
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .fn(async t => {
    const cases = await d.get(t.params.inputSource === 'const' ? 'f32_const' : 'f32_non_const');
    await run(t, builtin('quantizeToF16'), [TypeF32], TypeF32, t.params, cases);
  });
