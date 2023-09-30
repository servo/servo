/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Execution tests for the 'fract' builtin function

S is AbstractFloat, f32, f16
T is S or vecN<S>
@const fn fract(e: T ) -> T
Returns the fractional part of e, computed as e - floor(e).
Component-wise when T is a vector.
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';
import { TypeF32, TypeF16 } from '../../../../../util/conversion.js';
import { FP } from '../../../../../util/floating_point.js';
import { fullF32Range, fullF16Range } from '../../../../../util/math.js';
import { makeCaseCache } from '../../case_cache.js';
import { allInputSources, run } from '../../expression.js';

import { builtin } from './builtin.js';

export const g = makeTestGroup(GPUTest);

export const d = makeCaseCache('fract', {
  f32: () => {
    return FP.f32.generateScalarToIntervalCases(
      [
        0.5, // 0.5 -> 0.5
        0.9, // ~0.9 -> ~0.9
        1, // 1 -> 0
        2, // 2 -> 0
        1.11, // ~1.11 -> ~0.11
        10.0001, // ~10.0001 -> ~0.0001
        -0.1, // ~-0.1 -> ~0.9
        -0.5, // -0.5 -> 0.5
        -0.9, // ~-0.9 -> ~0.1
        -1, // -1 -> 0
        -2, // -2 -> 0
        -1.11, // ~-1.11 -> ~0.89
        -10.0001, // -10.0001 -> ~0.9999
        0x80000000, // https://github.com/gpuweb/cts/issues/2766
        ...fullF32Range(),
      ],

      'unfiltered',
      FP.f32.fractInterval
    );
  },
  f16: () => {
    return FP.f16.generateScalarToIntervalCases(
      [
        0.5, // 0.5 -> 0.5
        0.9, // ~0.9 -> ~0.9
        1, // 1 -> 0
        2, // 2 -> 0
        1.11, // ~1.11 -> ~0.11
        10.0078125, // 10.0078125 -> 0.0078125
        -0.1, // ~-0.1 -> ~0.9
        -0.5, // -0.5 -> 0.5
        -0.9, // ~-0.9 -> ~0.1
        -1, // -1 -> 0
        -2, // -2 -> 0
        -1.11, // ~-1.11 -> ~0.89
        -10.0078125, // -10.0078125 -> 0.9921875
        658.5, // 658.5 -> 0.5
        ...fullF16Range(),
      ],

      'unfiltered',
      FP.f16.fractInterval
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
    await run(t, builtin('fract'), [TypeF32], TypeF32, t.params, cases);
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
    await run(t, builtin('fract'), [TypeF16], TypeF16, t.params, cases);
  });
