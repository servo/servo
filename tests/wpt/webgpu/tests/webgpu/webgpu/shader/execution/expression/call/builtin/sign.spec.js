/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Execution tests for the 'sign' builtin function

S is AbstractFloat, AbstractInt, i32, f32, f16
T is S or vecN<S>
@const fn sign(e: T ) -> T
Returns the sign of e. Component-wise when T is a vector.
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';
import { i32, TypeF32, TypeF16, TypeI32 } from '../../../../../util/conversion.js';
import { FP } from '../../../../../util/floating_point.js';
import { fullF32Range, fullF16Range, fullI32Range } from '../../../../../util/math.js';
import { makeCaseCache } from '../../case_cache.js';
import { allInputSources, run } from '../../expression.js';

import { builtin } from './builtin.js';

export const g = makeTestGroup(GPUTest);

export const d = makeCaseCache('sign', {
  f32: () => {
    return FP.f32.generateScalarToIntervalCases(fullF32Range(), 'unfiltered', FP.f32.signInterval);
  },
  f16: () => {
    return FP.f16.generateScalarToIntervalCases(fullF16Range(), 'unfiltered', FP.f16.signInterval);
  },
  i32: () =>
    fullI32Range().map(i => {
      const signFunc = i => (i < 0 ? -1 : i > 0 ? 1 : 0);
      return { input: [i32(i)], expected: i32(signFunc(i)) };
    }),
});

g.test('abstract_float')
  .specURL('https://www.w3.org/TR/WGSL/#sign-builtin')
  .desc(`abstract float tests`)
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .unimplemented();

g.test('abstract_int')
  .specURL('https://www.w3.org/TR/WGSL/#sign-builtin')
  .desc(`abstract float tests`)
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .unimplemented();

g.test('i32')
  .specURL('https://www.w3.org/TR/WGSL/#sign-builtin')
  .desc(`i32 tests`)
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .fn(async t => {
    const cases = await d.get('i32');
    await run(t, builtin('sign'), [TypeI32], TypeI32, t.params, cases);
  });

g.test('f32')
  .specURL('https://www.w3.org/TR/WGSL/#sign-builtin')
  .desc(`f32 tests`)
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .fn(async t => {
    const cases = await d.get('f32');
    await run(t, builtin('sign'), [TypeF32], TypeF32, t.params, cases);
  });

g.test('f16')
  .specURL('https://www.w3.org/TR/WGSL/#sign-builtin')
  .desc(`f16 tests`)
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .beforeAllSubcases(t => {
    t.selectDeviceOrSkipTestCase('shader-f16');
  })
  .fn(async t => {
    const cases = await d.get('f16');
    await run(t, builtin('sign'), [TypeF16], TypeF16, t.params, cases);
  });
