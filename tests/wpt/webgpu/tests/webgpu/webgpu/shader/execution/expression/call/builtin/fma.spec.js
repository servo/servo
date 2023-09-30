/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Execution tests for the 'fma' builtin function

S is AbstractFloat, f32, f16
T is S or vecN<S>
@const fn fma(e1: T ,e2: T ,e3: T ) -> T
Returns e1 * e2 + e3. Component-wise when T is a vector.
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';
import { TypeF32, TypeF16, TypeAbstractFloat } from '../../../../../util/conversion.js';
import { FP } from '../../../../../util/floating_point.js';
import { sparseF32Range, sparseF16Range, sparseF64Range } from '../../../../../util/math.js';
import { makeCaseCache } from '../../case_cache.js';
import { allInputSources, onlyConstInputSource, run } from '../../expression.js';

import { abstractBuiltin, builtin } from './builtin.js';

export const g = makeTestGroup(GPUTest);

export const d = makeCaseCache('fma', {
  f32_const: () => {
    return FP.f32.generateScalarTripleToIntervalCases(
      sparseF32Range(),
      sparseF32Range(),
      sparseF32Range(),
      'finite',
      FP.f32.fmaInterval
    );
  },
  f32_non_const: () => {
    return FP.f32.generateScalarTripleToIntervalCases(
      sparseF32Range(),
      sparseF32Range(),
      sparseF32Range(),
      'unfiltered',
      FP.f32.fmaInterval
    );
  },
  f16_const: () => {
    return FP.f16.generateScalarTripleToIntervalCases(
      sparseF16Range(),
      sparseF16Range(),
      sparseF16Range(),
      'finite',
      FP.f16.fmaInterval
    );
  },
  f16_non_const: () => {
    return FP.f16.generateScalarTripleToIntervalCases(
      sparseF16Range(),
      sparseF16Range(),
      sparseF16Range(),
      'unfiltered',
      FP.f16.fmaInterval
    );
  },
  abstract: () => {
    return FP.abstract.generateScalarTripleToIntervalCases(
      sparseF64Range(),
      sparseF64Range(),
      sparseF64Range(),
      'finite',
      FP.abstract.fmaInterval
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
      abstractBuiltin('fma'),
      [TypeAbstractFloat, TypeAbstractFloat, TypeAbstractFloat],
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
    const cases = await d.get(t.params.inputSource === 'const' ? 'f32_const' : 'f32_non_const');
    await run(t, builtin('fma'), [TypeF32, TypeF32, TypeF32], TypeF32, t.params, cases);
  });

g.test('f16')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(`f16 tests`)
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .beforeAllSubcases(t => {
    t.selectDeviceOrSkipTestCase({ requiredFeatures: ['shader-f16'] });
  })
  .fn(async t => {
    const cases = await d.get(t.params.inputSource === 'const' ? 'f16_const' : 'f16_non_const');
    await run(t, builtin('fma'), [TypeF16, TypeF16, TypeF16], TypeF16, t.params, cases);
  });
