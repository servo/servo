/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Execution Tests for assignment of AbstractFloats to concrete data types
`;
import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../gpu_test.js';
import { kValue } from '../../../../util/constants.js';
import { abstractFloat, TypeAbstractFloat, TypeF16, TypeF32 } from '../../../../util/conversion.js';
import { FP } from '../../../../util/floating_point.js';
import { filteredF64Range } from '../../../../util/math.js';
import { makeCaseCache } from '../case_cache.js';
import { allInputSources, run } from '../expression.js';

import { assignment } from './unary.js';

export const g = makeTestGroup(GPUTest);

export const d = makeCaseCache('unary/abstract_float_assignment', {
  f32: () => {
    return filteredF64Range(kValue.f32.negative.min, kValue.f32.positive.max).map(f => {
      return { input: abstractFloat(f), expected: FP.f32.correctlyRoundedInterval(f) };
    });
  },
  f16: () => {
    return filteredF64Range(kValue.f16.negative.min, kValue.f16.positive.max).map(f => {
      return { input: abstractFloat(f), expected: FP.f16.correctlyRoundedInterval(f) };
    });
  },
});

g.test('f32')
  .specURL('https://www.w3.org/TR/WGSL/#floating-point-conversion')
  .desc(
    `
concretizing to f32
`
  )
  .params(u => u.combine('inputSource', [allInputSources[0]])) // Only defined for const-eval
  .fn(async t => {
    const cases = await d.get('f32');
    await run(t, assignment(), [TypeAbstractFloat], TypeF32, t.params, cases);
  });

g.test('f16')
  .specURL('https://www.w3.org/TR/WGSL/#floating-point-conversion')
  .desc(
    `
concretizing to f16
`
  )
  .beforeAllSubcases(t => {
    t.selectDeviceOrSkipTestCase({ requiredFeatures: ['shader-f16'] });
  })
  .params(u => u.combine('inputSource', [allInputSources[0]])) // Only defined for const-eval
  .fn(async t => {
    const cases = await d.get('f16');
    await run(t, assignment(), [TypeAbstractFloat], TypeF16, t.params, cases);
  });
