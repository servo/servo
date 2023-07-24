/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Execution tests for the 'smoothstep' builtin function

S is AbstractFloat, f32, f16
T is S or vecN<S>
@const fn smoothstep(low: T , high: T , x: T ) -> T
Returns the smooth Hermite interpolation between 0 and 1.
Component-wise when T is a vector.
For scalar T, the result is t * t * (3.0 - 2.0 * t), where t = clamp((x - low) / (high - low), 0.0, 1.0).
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';
import { TypeF32 } from '../../../../../util/conversion.js';
import { FP } from '../../../../../util/floating_point.js';
import { sparseF32Range } from '../../../../../util/math.js';
import { makeCaseCache } from '../../case_cache.js';
import { allInputSources, run } from '../../expression.js';

import { builtin } from './builtin.js';

export const g = makeTestGroup(GPUTest);

export const d = makeCaseCache('smoothstep', {
  f32_const: () => {
    return FP.f32.generateScalarTripleToIntervalCases(
      sparseF32Range(),
      sparseF32Range(),
      sparseF32Range(),
      'finite',
      FP.f32.smoothStepInterval
    );
  },
  f32_non_const: () => {
    return FP.f32.generateScalarTripleToIntervalCases(
      sparseF32Range(),
      sparseF32Range(),
      sparseF32Range(),
      'unfiltered',
      FP.f32.smoothStepInterval
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
    const cases = await d.get(t.params.inputSource === 'const' ? 'f32_const' : 'f32_non_const');
    await run(t, builtin('smoothstep'), [TypeF32, TypeF32, TypeF32], TypeF32, t.params, cases);
  });

g.test('f16')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(`f16 tests`)
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .unimplemented();
