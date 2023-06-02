/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Execution tests for the 'refract' builtin function

T is vecN<I>
I is AbstractFloat, f32, or f16
@const fn refract(e1: T ,e2: T ,e3: I ) -> T
For the incident vector e1 and surface normal e2, and the ratio of indices of
refraction e3, let k = 1.0 -e3*e3* (1.0 - dot(e2,e1) * dot(e2,e1)).
If k < 0.0, returns the refraction vector 0.0, otherwise return the refraction
vector e3*e1- (e3* dot(e2,e1) + sqrt(k)) *e2.
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';
import { toVector, TypeF32, TypeVec } from '../../../../../util/conversion.js';
import { FP } from '../../../../../util/floating_point.js';
import { sparseVectorF32Range, sparseF32Range } from '../../../../../util/math.js';
import { makeCaseCache } from '../../case_cache.js';
import { allInputSources, run } from '../../expression.js';

import { builtin } from './builtin.js';

export const g = makeTestGroup(GPUTest);

// Using a bespoke implementation of make*Case and generate*Cases here
// since refract is the only builtin with the API signature
// (vec, vec, scalar) -> vec

/**
 * @returns a Case for `refract`
 * @param kind what type of floating point numbers to operate on
 * @param i the `i` param for the case
 * @param s the `s` param for the case
 * @param r the `r` param for the case
 * @param check what interval checking to apply
 * */
function makeCase(kind, i, s, r, check) {
  const fp = FP[kind];
  i = i.map(fp.quantize);
  s = s.map(fp.quantize);
  r = fp.quantize(r);

  const vectors = fp.refractInterval(i, s, r);
  if (check === 'finite' && vectors.some(e => !e.isFinite())) {
    return undefined;
  }

  return {
    input: [toVector(i, fp.scalarBuilder), toVector(s, fp.scalarBuilder), fp.scalarBuilder(r)],
    expected: fp.refractInterval(i, s, r),
  };
}

/**
 * @returns an array of Cases for `refract`
 * @param kind what type of floating point numbers to operate on
 * @param param_is array of inputs to try for the `i` param
 * @param param_ss array of inputs to try for the `s` param
 * @param param_rs array of inputs to try for the `r` param
 * @param check what interval checking to apply
 */
function generateCases(kind, param_is, param_ss, param_rs, check) {
  // Cannot use `cartesianProduct` here due to heterogeneous param types
  return param_is
    .flatMap(i => {
      return param_ss.flatMap(s => {
        return param_rs.map(r => {
          return makeCase('f32', i, s, r, check);
        });
      });
    })
    .filter(c => c !== undefined);
}

export const d = makeCaseCache('refract', {
  f32_vec2_const: () => {
    return generateCases(
      'f32',
      sparseVectorF32Range(2),
      sparseVectorF32Range(2),
      sparseF32Range(),
      'finite'
    );
  },
  f32_vec2_non_const: () => {
    return generateCases(
      'f32',
      sparseVectorF32Range(2),
      sparseVectorF32Range(2),
      sparseF32Range(),
      'unfiltered'
    );
  },
  f32_vec3_const: () => {
    return generateCases(
      'f32',
      sparseVectorF32Range(3),
      sparseVectorF32Range(3),
      sparseF32Range(),
      'finite'
    );
  },
  f32_vec3_non_const: () => {
    return generateCases(
      'f32',
      sparseVectorF32Range(3),
      sparseVectorF32Range(3),
      sparseF32Range(),
      'unfiltered'
    );
  },
  f32_vec4_const: () => {
    return generateCases(
      'f32',
      sparseVectorF32Range(4),
      sparseVectorF32Range(4),
      sparseF32Range(),
      'finite'
    );
  },
  f32_vec4_non_const: () => {
    return generateCases(
      'f32',
      sparseVectorF32Range(4),
      sparseVectorF32Range(4),
      sparseF32Range(),
      'unfiltered'
    );
  },
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
      builtin('refract'),
      [TypeVec(2, TypeF32), TypeVec(2, TypeF32), TypeF32],
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
      builtin('refract'),
      [TypeVec(3, TypeF32), TypeVec(3, TypeF32), TypeF32],
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
      builtin('refract'),
      [TypeVec(4, TypeF32), TypeVec(4, TypeF32), TypeF32],
      TypeVec(4, TypeF32),
      t.params,
      cases
    );
  });

g.test('f16')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(`f16 tests`)
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [2, 3, 4]))
  .unimplemented();
