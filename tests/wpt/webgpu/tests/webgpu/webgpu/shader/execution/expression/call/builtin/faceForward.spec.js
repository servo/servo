/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Execution tests for the 'faceForward' builtin function

T is vecN<AbstractFloat>, vecN<f32>, or vecN<f16>
@const fn faceForward(e1: T ,e2: T ,e3: T ) -> T
Returns e1 if dot(e2,e3) is negative, and -e1 otherwise.
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';
import { anyOf } from '../../../../../util/compare.js';
import { toVector, TypeF32, TypeVec } from '../../../../../util/conversion.js';
import { FP } from '../../../../../util/floating_point.js';
import { cartesianProduct, sparseVectorF32Range } from '../../../../../util/math.js';
import { makeCaseCache } from '../../case_cache.js';
import { allInputSources, run } from '../../expression.js';

import { builtin } from './builtin.js';

export const g = makeTestGroup(GPUTest);

// Using a bespoke implementation of make*Case and generate*Cases here
// since faceForwardIntervals is the only builtin with the API signature
// (vec, vec, vec) -> vec
//
// Additionally faceForward has significant complexities around it due to the
// fact that `dot` is calculated in it s operation, but the result of dot isn't
// used to calculate the builtin's result.

/**
 * @returns a Case for `faceForward`
 * @param kind what kind of floating point numbers being operated on
 * @param x the `x` param for the case
 * @param y the `y` param for the case
 * @param z the `z` param for the case
 * @param check what interval checking to apply
 * */
function makeCase(kind, x, y, z, check) {
  const fp = FP[kind];
  x = x.map(fp.quantize);
  y = y.map(fp.quantize);
  z = z.map(fp.quantize);

  const results = FP.f32.faceForwardIntervals(x, y, z);
  if (check === 'finite' && results.some(r => r === undefined)) {
    return undefined;
  }

  // Stripping the undefined results, since undefined is used to signal that an OOB
  // could occur within the calculation that isn't reflected in the result
  // intervals.
  const define_results = results.filter(r => r !== undefined);

  return {
    input: [
      toVector(x, fp.scalarBuilder),
      toVector(y, fp.scalarBuilder),
      toVector(z, fp.scalarBuilder),
    ],

    expected: anyOf(...define_results),
  };
}

/**
 * @returns an array of Cases for `faceForward`
 * @param kind what kind of floating point numbers being operated on
 * @param xs array of inputs to try for the `x` param
 * @param ys array of inputs to try for the `y` param
 * @param zs array of inputs to try for the `z` param
 * @param check what interval checking to apply
 */
function generateCases(kind, xs, ys, zs, check) {
  // Cannot use `cartesianProduct` here due to heterogeneous param types
  return cartesianProduct(xs, ys, zs)
    .map(e => makeCase('f32', e[0], e[1], e[2], check))
    .filter(c => c !== undefined);
}

export const d = makeCaseCache('faceForward', {
  f32_vec2_const: () => {
    return generateCases(
      'f32',
      sparseVectorF32Range(2),
      sparseVectorF32Range(2),
      sparseVectorF32Range(2),
      'finite'
    );
  },
  f32_vec2_non_const: () => {
    return generateCases(
      'f32',
      sparseVectorF32Range(2),
      sparseVectorF32Range(2),
      sparseVectorF32Range(2),
      'unfiltered'
    );
  },
  f32_vec3_const: () => {
    return generateCases(
      'f32',
      sparseVectorF32Range(3),
      sparseVectorF32Range(3),
      sparseVectorF32Range(3),
      'finite'
    );
  },
  f32_vec3_non_const: () => {
    return generateCases(
      'f32',
      sparseVectorF32Range(3),
      sparseVectorF32Range(3),
      sparseVectorF32Range(3),
      'unfiltered'
    );
  },
  f32_vec4_const: () => {
    return generateCases(
      'f32',
      sparseVectorF32Range(4),
      sparseVectorF32Range(4),
      sparseVectorF32Range(4),
      'finite'
    );
  },
  f32_vec4_non_const: () => {
    return generateCases(
      'f32',
      sparseVectorF32Range(4),
      sparseVectorF32Range(4),
      sparseVectorF32Range(4),
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
      builtin('faceForward'),
      [TypeVec(2, TypeF32), TypeVec(2, TypeF32), TypeVec(2, TypeF32)],
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
      builtin('faceForward'),
      [TypeVec(3, TypeF32), TypeVec(3, TypeF32), TypeVec(3, TypeF32)],
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
      builtin('faceForward'),
      [TypeVec(4, TypeF32), TypeVec(4, TypeF32), TypeVec(4, TypeF32)],
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
