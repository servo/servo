/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Execution tests for the 'determinant' builtin function

T is AbstractFloat, f32, or f16
@const determinant(e: matCxC<T> ) -> T
Returns the determinant of e.
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';
import { TypeF32, TypeMat } from '../../../../../util/conversion.js';
import { FP } from '../../../../../util/floating_point.js';
import { makeCaseCache } from '../../case_cache.js';
import { allInputSources, run } from '../../expression.js';

import { builtin } from './builtin.js';

export const g = makeTestGroup(GPUTest);

// Accuracy for determinant is only defined for e, where e is an integer and
// |e| < quadroot(2**21) [~38],
// due to computational complexity of calculating the general solution for 4x4,
// so custom matrices are used.
//
// Note: For 2x2 and 3x3 the limits are squareroot and cuberoot instead of
// quadroot, but using the tighter 4x4 limits for all cases for simplicity.
const kDeterminantValues = [-38, -10, -5, -1, 0, 1, 5, 10, 38];

const kDeterminantMatrixF32Values = {
  2: kDeterminantValues.map((f, idx) => [
    [idx % 4 === 0 ? f : idx, idx % 4 === 1 ? f : -idx],
    [idx % 4 === 2 ? f : -idx, idx % 4 === 3 ? f : idx],
  ]),

  3: kDeterminantValues.map((f, idx) => [
    [idx % 9 === 0 ? f : idx, idx % 9 === 1 ? f : -idx, idx % 9 === 2 ? f : idx],
    [idx % 9 === 3 ? f : -idx, idx % 9 === 4 ? f : idx, idx % 9 === 5 ? f : -idx],
    [idx % 9 === 6 ? f : idx, idx % 9 === 7 ? f : -idx, idx % 9 === 8 ? f : idx],
  ]),

  4: kDeterminantValues.map((f, idx) => [
    [
      idx % 16 === 0 ? f : idx,
      idx % 16 === 1 ? f : -idx,
      idx % 16 === 2 ? f : idx,
      idx % 16 === 3 ? f : -idx,
    ],

    [
      idx % 16 === 4 ? f : -idx,
      idx % 16 === 5 ? f : idx,
      idx % 16 === 6 ? f : -idx,
      idx % 16 === 7 ? f : idx,
    ],

    [
      idx % 16 === 8 ? f : idx,
      idx % 16 === 9 ? f : -idx,
      idx % 16 === 10 ? f : idx,
      idx % 16 === 11 ? f : -idx,
    ],

    [
      idx % 16 === 12 ? f : -idx,
      idx % 16 === 13 ? f : idx,
      idx % 16 === 14 ? f : -idx,
      idx % 16 === 15 ? f : idx,
    ],
  ]),
};

export const d = makeCaseCache('determinant', {
  f32_mat2x2_const: () => {
    return FP.f32.generateMatrixToScalarCases(
      kDeterminantMatrixF32Values[2],
      'finite',
      FP.f32.determinantInterval
    );
  },
  f32_mat2x2_non_const: () => {
    return FP.f32.generateMatrixToScalarCases(
      kDeterminantMatrixF32Values[2],
      'unfiltered',
      FP.f32.determinantInterval
    );
  },
  f32_mat3x3_const: () => {
    return FP.f32.generateMatrixToScalarCases(
      kDeterminantMatrixF32Values[3],
      'finite',
      FP.f32.determinantInterval
    );
  },
  f32_mat3x3_non_const: () => {
    return FP.f32.generateMatrixToScalarCases(
      kDeterminantMatrixF32Values[3],
      'unfiltered',
      FP.f32.determinantInterval
    );
  },
  f32_mat4x4_const: () => {
    return FP.f32.generateMatrixToScalarCases(
      kDeterminantMatrixF32Values[4],
      'finite',
      FP.f32.determinantInterval
    );
  },
  f32_mat4x4_non_const: () => {
    return FP.f32.generateMatrixToScalarCases(
      kDeterminantMatrixF32Values[4],
      'unfiltered',
      FP.f32.determinantInterval
    );
  },
});

g.test('abstract_float')
  .specURL('https://www.w3.org/TR/WGSL/#matrix-builtin-functions')
  .desc(`abstract float tests`)
  .params(u => u.combine('inputSource', allInputSources).combine('dimension', [2, 3, 4]))
  .unimplemented();

g.test('f32')
  .specURL('https://www.w3.org/TR/WGSL/#matrix-builtin-functions')
  .desc(`f32 tests`)
  .params(u => u.combine('inputSource', allInputSources).combine('dim', [2, 3, 4]))
  .fn(async t => {
    const dim = t.params.dim;
    const cases = await d.get(
      t.params.inputSource === 'const'
        ? `f32_mat${dim}x${dim}_const`
        : `f32_mat${dim}x${dim}_non_const`
    );

    await run(t, builtin('determinant'), [TypeMat(dim, dim, TypeF32)], TypeF32, t.params, cases);
  });

g.test('f16')
  .specURL('https://www.w3.org/TR/WGSL/#matrix-builtin-functions')
  .desc(`f16 tests`)
  .params(u => u.combine('inputSource', allInputSources).combine('dimension', [2, 3, 4]))
  .unimplemented();
