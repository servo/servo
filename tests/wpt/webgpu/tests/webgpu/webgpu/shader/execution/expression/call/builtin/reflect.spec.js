/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Execution tests for the 'reflect' builtin function

T is vecN<AbstractFloat>, vecN<f32>, or vecN<f16>
@const fn reflect(e1: T, e2: T ) -> T
For the incident vector e1 and surface orientation e2, returns the reflection
direction e1-2*dot(e2,e1)*e2.
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';
import { TypeF32, TypeF16, TypeVec } from '../../../../../util/conversion.js';
import { FP } from '../../../../../util/floating_point.js';
import { sparseVectorF32Range, sparseVectorF16Range } from '../../../../../util/math.js';
import { makeCaseCache } from '../../case_cache.js';
import { allInputSources, run } from '../../expression.js';

import { builtin } from './builtin.js';

export const g = makeTestGroup(GPUTest);

// Cases: f32_vecN_[non_]const
const f32_vec_cases = [2, 3, 4]
  .flatMap(n =>
    [true, false].map(nonConst => ({
      [`f32_vec${n}_${nonConst ? 'non_const' : 'const'}`]: () => {
        return FP.f32.generateVectorPairToVectorCases(
          sparseVectorF32Range(n),
          sparseVectorF32Range(n),
          nonConst ? 'unfiltered' : 'finite',
          FP.f32.reflectInterval
        );
      },
    }))
  )
  .reduce((a, b) => ({ ...a, ...b }), {});

// Cases: f16_vecN_[non_]const
const f16_vec_cases = [2, 3, 4]
  .flatMap(n =>
    [true, false].map(nonConst => ({
      [`f16_vec${n}_${nonConst ? 'non_const' : 'const'}`]: () => {
        return FP.f16.generateVectorPairToVectorCases(
          sparseVectorF16Range(n),
          sparseVectorF16Range(n),
          nonConst ? 'unfiltered' : 'finite',
          FP.f16.reflectInterval
        );
      },
    }))
  )
  .reduce((a, b) => ({ ...a, ...b }), {});

export const d = makeCaseCache('reflect', {
  ...f32_vec_cases,
  ...f16_vec_cases,
});

g.test('abstract_float')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(`abstract float tests`)
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [2, 3, 4]))
  .unimplemented();

g.test('f32_vec2')
  .specURL('https://www.w3.org/TR/WGSL/#numeric-builtin-functions')
  .desc(`f32 tests using vec2s`)
  .params(u => u.combine('inputSource', allInputSources))
  .fn(async t => {
    const cases = await d.get(
      t.params.inputSource === 'const' ? 'f32_vec2_const' : 'f32_vec2_non_const'
    );

    await run(
      t,
      builtin('reflect'),
      [TypeVec(2, TypeF32), TypeVec(2, TypeF32)],
      TypeVec(2, TypeF32),
      t.params,
      cases
    );
  });

g.test('f32_vec3')
  .specURL('https://www.w3.org/TR/WGSL/#numeric-builtin-functions')
  .desc(`f32 tests using vec3s`)
  .params(u => u.combine('inputSource', allInputSources))
  .fn(async t => {
    const cases = await d.get(
      t.params.inputSource === 'const' ? 'f32_vec3_const' : 'f32_vec3_non_const'
    );

    await run(
      t,
      builtin('reflect'),
      [TypeVec(3, TypeF32), TypeVec(3, TypeF32)],
      TypeVec(3, TypeF32),
      t.params,
      cases
    );
  });

g.test('f32_vec4')
  .specURL('https://www.w3.org/TR/WGSL/#numeric-builtin-functions')
  .desc(`f32 tests using vec4s`)
  .params(u => u.combine('inputSource', allInputSources))
  .fn(async t => {
    const cases = await d.get(
      t.params.inputSource === 'const' ? 'f32_vec4_const' : 'f32_vec4_non_const'
    );

    await run(
      t,
      builtin('reflect'),
      [TypeVec(4, TypeF32), TypeVec(4, TypeF32)],
      TypeVec(4, TypeF32),
      t.params,
      cases
    );
  });

g.test('f16_vec2')
  .specURL('https://www.w3.org/TR/WGSL/#numeric-builtin-functions')
  .desc(`f16 tests using vec2s`)
  .params(u => u.combine('inputSource', allInputSources))
  .beforeAllSubcases(t => {
    t.selectDeviceOrSkipTestCase('shader-f16');
  })
  .fn(async t => {
    const cases = await d.get(
      t.params.inputSource === 'const' ? 'f16_vec2_const' : 'f16_vec2_non_const'
    );

    await run(
      t,
      builtin('reflect'),
      [TypeVec(2, TypeF16), TypeVec(2, TypeF16)],
      TypeVec(2, TypeF16),
      t.params,
      cases
    );
  });

g.test('f16_vec3')
  .specURL('https://www.w3.org/TR/WGSL/#numeric-builtin-functions')
  .desc(`f16 tests using vec3s`)
  .params(u => u.combine('inputSource', allInputSources))
  .beforeAllSubcases(t => {
    t.selectDeviceOrSkipTestCase('shader-f16');
  })
  .fn(async t => {
    const cases = await d.get(
      t.params.inputSource === 'const' ? 'f16_vec3_const' : 'f16_vec3_non_const'
    );

    await run(
      t,
      builtin('reflect'),
      [TypeVec(3, TypeF16), TypeVec(3, TypeF16)],
      TypeVec(3, TypeF16),
      t.params,
      cases
    );
  });

g.test('f16_vec4')
  .specURL('https://www.w3.org/TR/WGSL/#numeric-builtin-functions')
  .desc(`f16 tests using vec4s`)
  .params(u => u.combine('inputSource', allInputSources))
  .beforeAllSubcases(t => {
    t.selectDeviceOrSkipTestCase('shader-f16');
  })
  .fn(async t => {
    const cases = await d.get(
      t.params.inputSource === 'const' ? 'f16_vec4_const' : 'f16_vec4_non_const'
    );

    await run(
      t,
      builtin('reflect'),
      [TypeVec(4, TypeF16), TypeVec(4, TypeF16)],
      TypeVec(4, TypeF16),
      t.params,
      cases
    );
  });
