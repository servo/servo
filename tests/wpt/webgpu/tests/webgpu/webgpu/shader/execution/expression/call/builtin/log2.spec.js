/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Execution tests for the 'log2' builtin function

S is AbstractFloat, f32, f16
T is S or vecN<S>
@const fn log2(e: T ) -> T
Returns the base-2 logarithm of e. Component-wise when T is a vector.
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';
import { kValue } from '../../../../../util/constants.js';
import { TypeF32 } from '../../../../../util/conversion.js';
import { FP } from '../../../../../util/floating_point.js';
import { biasedRange, fullF32Range, linearRange } from '../../../../../util/math.js';
import { makeCaseCache } from '../../case_cache.js';
import { allInputSources, run } from '../../expression.js';

import { builtin } from './builtin.js';

export const g = makeTestGroup(GPUTest);

// log2's accuracy is defined in three regions { [0, 0.5), [0.5, 2.0], (2.0, +∞] }
const inputs = [
  ...linearRange(kValue.f32.positive.min, 0.5, 20),
  ...linearRange(0.5, 2.0, 20),
  ...biasedRange(2.0, 2 ** 32, 1000),
  ...fullF32Range(),
];

export const d = makeCaseCache('log2', {
  f32_const: () => {
    return FP.f32.generateScalarToIntervalCases(inputs, 'finite', FP.f32.log2Interval);
  },
  f32_non_const: () => {
    return FP.f32.generateScalarToIntervalCases(inputs, 'unfiltered', FP.f32.log2Interval);
  },
});

g.test('abstract_float')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(`abstract float tests`)
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .unimplemented();

g.test('f32')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(
    `
f32 tests

TODO(#792): Decide what the ground-truth is for these tests. [1]
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .fn(async t => {
    const cases = await d.get(t.params.inputSource === 'const' ? 'f32_const' : 'f32_non_const');
    await run(t, builtin('log2'), [TypeF32], TypeF32, t.params, cases);
  });

g.test('f16')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(`f16 tests`)
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .unimplemented();
