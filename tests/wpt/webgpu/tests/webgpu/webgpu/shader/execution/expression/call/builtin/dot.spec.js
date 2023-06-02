/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Execution tests for the 'dot' builtin function

T is AbstractInt, AbstractFloat, i32, u32, f32, or f16
@const fn dot(e1: vecN<T>,e2: vecN<T>) -> T
Returns the dot product of e1 and e2.
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';
import { TypeF32, TypeVec } from '../../../../../util/conversion.js';
import { FP } from '../../../../../util/floating_point.js';
import { sparseVectorF32Range, vectorF32Range } from '../../../../../util/math.js';
import { makeCaseCache } from '../../case_cache.js';
import { allInputSources, run } from '../../expression.js';

import { builtin } from './builtin.js';

export const g = makeTestGroup(GPUTest);

// vec3 and vec4 require calculating all possible permutations, so their runtime is much longer per test, so only using
// sparse vectors for them
export const d = makeCaseCache('dot', {
  f32_vec2_const: () => {
    return FP.f32.generateVectorPairToIntervalCases(
      vectorF32Range(2),
      vectorF32Range(2),
      'finite',
      FP.f32.dotInterval
    );
  },
  f32_vec2_non_const: () => {
    return FP.f32.generateVectorPairToIntervalCases(
      vectorF32Range(2),
      vectorF32Range(2),
      'unfiltered',
      FP.f32.dotInterval
    );
  },
  f32_vec3_const: () => {
    return FP.f32.generateVectorPairToIntervalCases(
      sparseVectorF32Range(3),
      sparseVectorF32Range(3),
      'finite',
      FP.f32.dotInterval
    );
  },
  f32_vec3_non_const: () => {
    return FP.f32.generateVectorPairToIntervalCases(
      sparseVectorF32Range(3),
      sparseVectorF32Range(3),
      'unfiltered',
      FP.f32.dotInterval
    );
  },
  f32_vec4_const: () => {
    return FP.f32.generateVectorPairToIntervalCases(
      sparseVectorF32Range(4),
      sparseVectorF32Range(4),
      'finite',
      FP.f32.dotInterval
    );
  },
  f32_vec4_non_const: () => {
    return FP.f32.generateVectorPairToIntervalCases(
      sparseVectorF32Range(4),
      sparseVectorF32Range(4),
      'unfiltered',
      FP.f32.dotInterval
    );
  },
});

g.test('abstract_int')
  .specURL('https://www.w3.org/TR/WGSL/#vector-builtin-functions')
  .desc(`abstract int tests`)
  .params(u => u.combine('inputSource', allInputSources))
  .unimplemented();

g.test('i32')
  .specURL('https://www.w3.org/TR/WGSL/#vector-builtin-functions')
  .desc(`i32 tests`)
  .params(u => u.combine('inputSource', allInputSources))
  .unimplemented();

g.test('u32')
  .specURL('https://www.w3.org/TR/WGSL/#vector-builtin-functions')
  .desc(`u32 tests`)
  .params(u => u.combine('inputSource', allInputSources))
  .unimplemented();

g.test('abstract_float')
  .specURL('https://www.w3.org/TR/WGSL/#vector-builtin-functions')
  .desc(`abstract float test`)
  .params(u => u.combine('inputSource', allInputSources))
  .unimplemented();

g.test('f32_vec2')
  .specURL('https://www.w3.org/TR/WGSL/#vector-builtin-functions')
  .desc(`f32 tests using vec2s`)
  .params(u => u.combine('inputSource', allInputSources))
  .fn(async t => {
    const cases = await d.get(
      t.params.inputSource === 'const' ? 'f32_vec2_const' : 'f32_vec2_non_const'
    );

    await run(
      t,
      builtin('dot'),
      [TypeVec(2, TypeF32), TypeVec(2, TypeF32)],
      TypeF32,
      t.params,
      cases
    );
  });

g.test('f32_vec3')
  .specURL('https://www.w3.org/TR/WGSL/#vector-builtin-functions')
  .desc(`f32 tests using vec3s`)
  .params(u => u.combine('inputSource', allInputSources))
  .fn(async t => {
    const cases = await d.get(
      t.params.inputSource === 'const' ? 'f32_vec3_const' : 'f32_vec3_non_const'
    );

    await run(
      t,
      builtin('dot'),
      [TypeVec(3, TypeF32), TypeVec(3, TypeF32)],
      TypeF32,
      t.params,
      cases
    );
  });

g.test('f32_vec4')
  .specURL('https://www.w3.org/TR/WGSL/#vector-builtin-functions')
  .desc(`f32 tests using vec4s`)
  .params(u => u.combine('inputSource', allInputSources))
  .fn(async t => {
    const cases = await d.get(
      t.params.inputSource === 'const' ? 'f32_vec4_const' : 'f32_vec4_non_const'
    );

    await run(
      t,
      builtin('dot'),
      [TypeVec(4, TypeF32), TypeVec(4, TypeF32)],
      TypeF32,
      t.params,
      cases
    );
  });

g.test('f16')
  .specURL('https://www.w3.org/TR/WGSL/#vector-builtin-functions')
  .desc(`f16 tests`)
  .params(u => u.combine('inputSource', allInputSources))
  .unimplemented();
