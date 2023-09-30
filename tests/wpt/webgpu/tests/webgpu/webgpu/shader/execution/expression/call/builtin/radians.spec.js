/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Execution tests for the 'radians' builtin function

S is AbstractFloat, f32, f16
T is S or vecN<S>
@const fn radians(e1: T ) -> T
Converts degrees to radians, approximating e1 * π / 180.
Component-wise when T is a vector
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';
import { TypeAbstractFloat, TypeF16, TypeF32 } from '../../../../../util/conversion.js';
import { FP } from '../../../../../util/floating_point.js';
import { fullF16Range, fullF32Range } from '../../../../../util/math.js';
import { makeCaseCache } from '../../case_cache.js';
import { allInputSources, onlyConstInputSource, run } from '../../expression.js';

import { abstractBuiltin, builtin } from './builtin.js';

export const g = makeTestGroup(GPUTest);

export const d = makeCaseCache('radians', {
  f32: () => {
    return FP.f32.generateScalarToIntervalCases(
      fullF32Range(),
      'unfiltered',
      FP.f32.radiansInterval
    );
  },
  f16: () => {
    return FP.f16.generateScalarToIntervalCases(
      fullF16Range(),
      'unfiltered',
      FP.f16.radiansInterval
    );
  },
  abstract: () => {
    return FP.abstract.generateScalarToIntervalCases(
      fullF16Range(),
      'unfiltered',
      FP.abstract.radiansInterval
    );
  },
});

g.test('abstract_float')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(`abstract float tests`)
  .params(u =>
    u.combine('inputSource', onlyConstInputSource).combine('vectorize', [undefined, 2, 3, 4])
  )
  .fn(async t => {
    const cases = await d.get('abstract');
    await run(
      t,
      abstractBuiltin('radians'),
      [TypeAbstractFloat],
      TypeAbstractFloat,
      t.params,
      cases
    );
  });

g.test('f32')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(`f32 tests`)
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .fn(async t => {
    const cases = await d.get('f32');
    await run(t, builtin('radians'), [TypeF32], TypeF32, t.params, cases);
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
    await run(t, builtin('radians'), [TypeF16], TypeF16, t.params, cases);
  });
