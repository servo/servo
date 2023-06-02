/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Execution tests for the 'length' builtin function

S is AbstractFloat, f32, f16
T is S or vecN<S>
@const fn length(e: T ) -> f32
Returns the length of e (e.g. abs(e) if T is a scalar, or sqrt(e[0]^2 + e[1]^2 + ...) if T is a vector).
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';
import { TypeF32, TypeVec } from '../../../../../util/conversion.js';
import { FP } from '../../../../../util/floating_point.js';
import { fullF32Range, vectorF32Range } from '../../../../../util/math.js';
import { makeCaseCache } from '../../case_cache.js';
import { allInputSources, run } from '../../expression.js';

import { builtin } from './builtin.js';

export const g = makeTestGroup(GPUTest);

export const d = makeCaseCache('length', {
  f32: () => {
    return FP.f32.generateScalarToIntervalCases(
      fullF32Range(),
      'unfiltered',
      FP.f32.lengthInterval
    );
  },
  f32_vec2_const: () => {
    return FP.f32.generateVectorToIntervalCases(vectorF32Range(2), 'finite', FP.f32.lengthInterval);
  },
  f32_vec2_non_const: () => {
    return FP.f32.generateVectorToIntervalCases(
      vectorF32Range(2),
      'unfiltered',
      FP.f32.lengthInterval
    );
  },
  f32_vec3_const: () => {
    return FP.f32.generateVectorToIntervalCases(vectorF32Range(3), 'finite', FP.f32.lengthInterval);
  },
  f32_vec3_non_const: () => {
    return FP.f32.generateVectorToIntervalCases(
      vectorF32Range(3),
      'unfiltered',
      FP.f32.lengthInterval
    );
  },
  f32_vec4_const: () => {
    return FP.f32.generateVectorToIntervalCases(vectorF32Range(4), 'finite', FP.f32.lengthInterval);
  },
  f32_vec4_non_const: () => {
    return FP.f32.generateVectorToIntervalCases(
      vectorF32Range(4),
      'unfiltered',
      FP.f32.lengthInterval
    );
  },
});

g.test('abstract_float')
  .specURL('https://www.w3.org/TR/WGSL/#numeric-builtin-functions')
  .desc(`abstract float tests`)
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .unimplemented();

g.test('f32')
  .specURL('https://www.w3.org/TR/WGSL/#numeric-builtin-functions')
  .desc(`f32 tests`)
  .params(u => u.combine('inputSource', allInputSources))
  .fn(async t => {
    const cases = await d.get('f32');
    await run(t, builtin('length'), [TypeF32], TypeF32, t.params, cases);
  });

g.test('f32_vec2')
  .specURL('https://www.w3.org/TR/WGSL/#numeric-builtin-functions')
  .desc(`f32 tests using vec2s`)
  .params(u => u.combine('inputSource', allInputSources))
  .fn(async t => {
    const cases = await d.get(
      t.params.inputSource === 'const' ? 'f32_vec2_const' : 'f32_vec2_non_const'
    );

    await run(t, builtin('length'), [TypeVec(2, TypeF32)], TypeF32, t.params, cases);
  });

g.test('f32_vec3')
  .specURL('https://www.w3.org/TR/WGSL/#numeric-builtin-functions')
  .desc(`f32 tests using vec3s`)
  .params(u => u.combine('inputSource', allInputSources))
  .fn(async t => {
    const cases = await d.get(
      t.params.inputSource === 'const' ? 'f32_vec3_const' : 'f32_vec3_non_const'
    );

    await run(t, builtin('length'), [TypeVec(3, TypeF32)], TypeF32, t.params, cases);
  });

g.test('f32_vec4')
  .specURL('https://www.w3.org/TR/WGSL/#numeric-builtin-functions')
  .desc(`f32 tests using vec4s`)
  .params(u => u.combine('inputSource', allInputSources))
  .fn(async t => {
    const cases = await d.get(
      t.params.inputSource === 'const' ? 'f32_vec4_const' : 'f32_vec4_non_const'
    );

    await run(t, builtin('length'), [TypeVec(4, TypeF32)], TypeF32, t.params, cases);
  });

g.test('f16')
  .specURL('https://www.w3.org/TR/WGSL/#numeric-builtin-functions')
  .desc(`f16 tests`)
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .unimplemented();
