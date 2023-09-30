/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Execution tests for the 'cos' builtin function

S is AbstractFloat, f32, f16
T is S or vecN<S>
@const fn cos(e: T ) -> T
Returns the cosine of e. Component-wise when T is a vector.
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';
import { TypeF32, TypeF16 } from '../../../../../util/conversion.js';
import { FP } from '../../../../../util/floating_point.js';
import { fullF32Range, fullF16Range, linearRange } from '../../../../../util/math.js';
import { makeCaseCache } from '../../case_cache.js';
import { allInputSources, run } from '../../expression.js';

import { builtin } from './builtin.js';

export const g = makeTestGroup(GPUTest);

export const d = makeCaseCache('cos', {
  f32: () => {
    return FP.f32.generateScalarToIntervalCases(
      [
        // Well-defined accuracy range
        ...linearRange(-Math.PI, Math.PI, 1000),
        ...fullF32Range(),
      ],

      'unfiltered',
      FP.f32.cosInterval
    );
  },
  f16: () => {
    return FP.f16.generateScalarToIntervalCases(
      [
        // Well-defined accuracy range
        ...linearRange(-Math.PI, Math.PI, 1000),
        ...fullF16Range(),
      ],

      'unfiltered',
      FP.f16.cosInterval
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
  .desc(
    `
f32 tests

TODO(#792): Decide what the ground-truth is for these tests. [1]
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .fn(async t => {
    const cases = await d.get('f32');
    await run(t, builtin('cos'), [TypeF32], TypeF32, t.params, cases);
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
    await run(t, builtin('cos'), [TypeF16], TypeF16, t.params, cases);
  });
