/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Execution tests for the 'mix' builtin function

S is AbstractFloat, f32, f16
T is S or vecN<S>
@const fn mix(e1: T, e2: T, e3: T) -> T
Returns the linear blend of e1 and e2 (e.g. e1*(1-e3)+e2*e3). Component-wise when T is a vector.

T is AbstractFloat, f32, or f16
T2 is vecN<T>
@const fn mix(e1: T2, e2: T2, e3: T) -> T2
Returns the component-wise linear blend of e1 and e2, using scalar blending factor e3 for each component.
Same as mix(e1,e2,T2(e3)).

`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';
import { TypeVec, TypeF32, TypeF16 } from '../../../../../util/conversion.js';
import { FP } from '../../../../../util/floating_point.js';
import {
  sparseF32Range,
  sparseF16Range,
  sparseVectorF32Range,
  sparseVectorF16Range,
} from '../../../../../util/math.js';
import { makeCaseCache } from '../../case_cache.js';
import { allInputSources, run } from '../../expression.js';

import { builtin } from './builtin.js';

export const g = makeTestGroup(GPUTest);

// Cases: f32_vecN_scalar_[non_]const
const f32_vec_scalar_cases = [2, 3, 4]
  .flatMap(n =>
    [true, false].map(nonConst => ({
      [`f32_vec${n}_scalar_${nonConst ? 'non_const' : 'const'}`]: () => {
        return FP.f32.generateVectorPairScalarToVectorComponentWiseCase(
          sparseVectorF32Range(n),
          sparseVectorF32Range(n),
          sparseF32Range(),
          nonConst ? 'unfiltered' : 'finite',
          ...FP.f32.mixIntervals
        );
      },
    }))
  )
  .reduce((a, b) => ({ ...a, ...b }), {});

// Cases: f16_vecN_scalar_[non_]const
const f16_vec_scalar_cases = [2, 3, 4]
  .flatMap(n =>
    [true, false].map(nonConst => ({
      [`f16_vec${n}_scalar_${nonConst ? 'non_const' : 'const'}`]: () => {
        return FP.f16.generateVectorPairScalarToVectorComponentWiseCase(
          sparseVectorF16Range(n),
          sparseVectorF16Range(n),
          sparseF16Range(),
          nonConst ? 'unfiltered' : 'finite',
          ...FP.f16.mixIntervals
        );
      },
    }))
  )
  .reduce((a, b) => ({ ...a, ...b }), {});

export const d = makeCaseCache('mix', {
  f32_const: () => {
    return FP.f32.generateScalarTripleToIntervalCases(
      sparseF32Range(),
      sparseF32Range(),
      sparseF32Range(),
      'finite',
      ...FP.f32.mixIntervals
    );
  },
  f32_non_const: () => {
    return FP.f32.generateScalarTripleToIntervalCases(
      sparseF32Range(),
      sparseF32Range(),
      sparseF32Range(),
      'unfiltered',
      ...FP.f32.mixIntervals
    );
  },
  ...f32_vec_scalar_cases,
  f16_const: () => {
    return FP.f16.generateScalarTripleToIntervalCases(
      sparseF16Range(),
      sparseF16Range(),
      sparseF16Range(),
      'finite',
      ...FP.f16.mixIntervals
    );
  },
  f16_non_const: () => {
    return FP.f16.generateScalarTripleToIntervalCases(
      sparseF16Range(),
      sparseF16Range(),
      sparseF16Range(),
      'unfiltered',
      ...FP.f16.mixIntervals
    );
  },
  ...f16_vec_scalar_cases,
});

g.test('abstract_float_matching')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(`abstract_float test with matching third param`)
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .unimplemented();

g.test('abstract_float_nonmatching_vec2')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(`abstract_float tests with two vec2<abstract_float> params and scalar third param`)
  .params(u => u.combine('inputSource', allInputSources))
  .unimplemented();

g.test('abstract_float_nonmatching_vec3')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(`abstract_float tests with two vec3<abstract_float> params and scalar third param`)
  .params(u => u.combine('inputSource', allInputSources))
  .unimplemented();

g.test('abstract_float_nonmatching_vec4')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(`abstract_float tests with two vec4<abstract_float> params and scalar third param`)
  .params(u => u.combine('inputSource', allInputSources))
  .unimplemented();

g.test('f32_matching')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(`f32 test with matching third param`)
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .fn(async t => {
    const cases = await d.get(t.params.inputSource === 'const' ? 'f32_const' : 'f32_non_const');
    await run(t, builtin('mix'), [TypeF32, TypeF32, TypeF32], TypeF32, t.params, cases);
  });

g.test('f32_nonmatching_vec2')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(`f32 tests with two vec2<f32> params and scalar third param`)
  .params(u => u.combine('inputSource', allInputSources))
  .fn(async t => {
    const cases = await d.get(
      t.params.inputSource === 'const' ? 'f32_vec2_scalar_const' : 'f32_vec2_scalar_non_const'
    );

    await run(
      t,
      builtin('mix'),
      [TypeVec(2, TypeF32), TypeVec(2, TypeF32), TypeF32],
      TypeVec(2, TypeF32),
      t.params,
      cases
    );
  });

g.test('f32_nonmatching_vec3')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(`f32 tests with two vec3<f32> params and scalar third param`)
  .params(u => u.combine('inputSource', allInputSources))
  .fn(async t => {
    const cases = await d.get(
      t.params.inputSource === 'const' ? 'f32_vec3_scalar_const' : 'f32_vec3_scalar_non_const'
    );

    await run(
      t,
      builtin('mix'),
      [TypeVec(3, TypeF32), TypeVec(3, TypeF32), TypeF32],
      TypeVec(3, TypeF32),
      t.params,
      cases
    );
  });

g.test('f32_nonmatching_vec4')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(`f32 tests with two vec4<f32> params and scalar third param`)
  .params(u => u.combine('inputSource', allInputSources))
  .fn(async t => {
    const cases = await d.get(
      t.params.inputSource === 'const' ? 'f32_vec4_scalar_const' : 'f32_vec4_scalar_non_const'
    );

    await run(
      t,
      builtin('mix'),
      [TypeVec(4, TypeF32), TypeVec(4, TypeF32), TypeF32],
      TypeVec(4, TypeF32),
      t.params,
      cases
    );
  });

g.test('f16_matching')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(`f16 test with matching third param`)
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .beforeAllSubcases(t => {
    t.selectDeviceOrSkipTestCase('shader-f16');
  })
  .fn(async t => {
    const cases = await d.get(t.params.inputSource === 'const' ? 'f16_const' : 'f16_non_const');
    await run(t, builtin('mix'), [TypeF16, TypeF16, TypeF16], TypeF16, t.params, cases);
  });

g.test('f16_nonmatching_vec2')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(`f16 tests with two vec2<f16> params and scalar third param`)
  .params(u => u.combine('inputSource', allInputSources))
  .beforeAllSubcases(t => {
    t.selectDeviceOrSkipTestCase('shader-f16');
  })
  .fn(async t => {
    const cases = await d.get(
      t.params.inputSource === 'const' ? 'f16_vec2_scalar_const' : 'f16_vec2_scalar_non_const'
    );

    await run(
      t,
      builtin('mix'),
      [TypeVec(2, TypeF16), TypeVec(2, TypeF16), TypeF16],
      TypeVec(2, TypeF16),
      t.params,
      cases
    );
  });

g.test('f16_nonmatching_vec3')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(`f16 tests with two vec3<f16> params and scalar third param`)
  .params(u => u.combine('inputSource', allInputSources))
  .beforeAllSubcases(t => {
    t.selectDeviceOrSkipTestCase('shader-f16');
  })
  .fn(async t => {
    const cases = await d.get(
      t.params.inputSource === 'const' ? 'f16_vec3_scalar_const' : 'f16_vec3_scalar_non_const'
    );

    await run(
      t,
      builtin('mix'),
      [TypeVec(3, TypeF16), TypeVec(3, TypeF16), TypeF16],
      TypeVec(3, TypeF16),
      t.params,
      cases
    );
  });

g.test('f16_nonmatching_vec4')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(`f16 tests with two vec4<f16> params and scalar third param`)
  .params(u => u.combine('inputSource', allInputSources))
  .beforeAllSubcases(t => {
    t.selectDeviceOrSkipTestCase('shader-f16');
  })
  .fn(async t => {
    const cases = await d.get(
      t.params.inputSource === 'const' ? 'f16_vec4_scalar_const' : 'f16_vec4_scalar_non_const'
    );

    await run(
      t,
      builtin('mix'),
      [TypeVec(4, TypeF16), TypeVec(4, TypeF16), TypeF16],
      TypeVec(4, TypeF16),
      t.params,
      cases
    );
  });
