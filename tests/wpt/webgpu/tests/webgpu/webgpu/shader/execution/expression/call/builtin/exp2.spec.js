/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Execution tests for the 'exp2' builtin function

S is AbstractFloat, f32, f16
T is S or vecN<S>
@const fn exp2(e: T ) -> T
Returns 2 raised to the power e (e.g. 2^e). Component-wise when T is a vector.
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';
import { kValue } from '../../../../../util/constants.js';
import { TypeF32 } from '../../../../../util/conversion.js';
import { FP } from '../../../../../util/floating_point.js';
import { biasedRange, linearRange } from '../../../../../util/math.js';
import { makeCaseCache } from '../../case_cache.js';
import { allInputSources, run } from '../../expression.js';

import { builtin } from './builtin.js';

export const g = makeTestGroup(GPUTest);

// floor(log2(max f32 value)) = 127, so exp2(127) will be within range of a f32, but exp2(128) will not
// floor(ln(max f64 value)) = 1023, so exp2(1023) can be handled by the testing framework, but exp2(1024) will misbehave
const inputs = [
  0, // Returns 1 by definition
  -128, // Returns subnormal value
  kValue.f32.negative.min, // Closest to returning 0 as possible
  ...biasedRange(kValue.f32.negative.max, -127, 100),
  ...biasedRange(kValue.f32.positive.min, 127, 100),
  ...linearRange(128, 1023, 10), // Overflows f32, but not f64
];

export const d = makeCaseCache('exp2', {
  f32_const: () => {
    return FP.f32.generateScalarToIntervalCases(inputs, 'finite', FP.f32.exp2Interval);
  },
  f32_non_const: () => {
    return FP.f32.generateScalarToIntervalCases(inputs, 'unfiltered', FP.f32.exp2Interval);
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
    const cases = await d.get(t.params.inputSource === 'const' ? 'f32_const' : 'f32_non_const');
    await run(t, builtin('exp2'), [TypeF32], TypeF32, t.params, cases);
  });

g.test('f16')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(`f16 tests`)
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .unimplemented();
