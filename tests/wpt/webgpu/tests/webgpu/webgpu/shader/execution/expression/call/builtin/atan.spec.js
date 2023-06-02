/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Execution tests for the 'atan' builtin function

S is AbstractFloat, f32, f16
T is S or vecN<S>
@const fn atan(e: T ) -> T
Returns the arc tangent of e. Component-wise when T is a vector.

`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';
import { TypeF32 } from '../../../../../util/conversion.js';
import { FP } from '../../../../../util/floating_point.js';
import { fullF32Range } from '../../../../../util/math.js';
import { makeCaseCache } from '../../case_cache.js';
import { allInputSources, run } from '../../expression.js';

import { builtin } from './builtin.js';

export const g = makeTestGroup(GPUTest);

const inputs = [
  // Known values
  -Math.sqrt(3),
  -1,
  -1 / Math.sqrt(3),
  0,
  1,
  1 / Math.sqrt(3),
  Math.sqrt(3),

  ...fullF32Range(),
];

export const d = makeCaseCache('atan', {
  f32_const: () => {
    return FP.f32.generateScalarToIntervalCases(inputs, 'finite', FP.f32.atanInterval);
  },
  f32_non_const: () => {
    return FP.f32.generateScalarToIntervalCases(inputs, 'unfiltered', FP.f32.atanInterval);
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
    await run(t, builtin('atan'), [TypeF32], TypeF32, t.params, cases);
  });

g.test('f16')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(`f16 tests`)
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .unimplemented();
