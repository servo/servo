/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { FP } from '../../../../../util/floating_point.js';import { makeCaseCache } from '../../case_cache.js';
// Accuracy for determinant is only defined for e, where e is an integer and
// |e| < quadroot(2**21) [~38],
// due to computational complexity of calculating the general solution for 4x4,
// so custom matrices are used.
//
// Note: For 2x2 and 3x3 the limits are squareroot and cuberoot instead of
// quadroot, but using the tighter 4x4 limits for all cases for simplicity.
const kDeterminantValues = [-38, -10, -5, -1, 0, 1, 5, 10, 38];

const kDeterminantMatrixValues = {
  2: kDeterminantValues.map((f, idx) => [
  [idx % 4 === 0 ? f : idx, idx % 4 === 1 ? f : -idx],
  [idx % 4 === 2 ? f : -idx, idx % 4 === 3 ? f : idx]]
  ),
  3: kDeterminantValues.map((f, idx) => [
  [idx % 9 === 0 ? f : idx, idx % 9 === 1 ? f : -idx, idx % 9 === 2 ? f : idx],
  [idx % 9 === 3 ? f : -idx, idx % 9 === 4 ? f : idx, idx % 9 === 5 ? f : -idx],
  [idx % 9 === 6 ? f : idx, idx % 9 === 7 ? f : -idx, idx % 9 === 8 ? f : idx]]
  ),
  4: kDeterminantValues.map((f, idx) => [
  [
  idx % 16 === 0 ? f : idx,
  idx % 16 === 1 ? f : -idx,
  idx % 16 === 2 ? f : idx,
  idx % 16 === 3 ? f : -idx],

  [
  idx % 16 === 4 ? f : -idx,
  idx % 16 === 5 ? f : idx,
  idx % 16 === 6 ? f : -idx,
  idx % 16 === 7 ? f : idx],

  [
  idx % 16 === 8 ? f : idx,
  idx % 16 === 9 ? f : -idx,
  idx % 16 === 10 ? f : idx,
  idx % 16 === 11 ? f : -idx],

  [
  idx % 16 === 12 ? f : -idx,
  idx % 16 === 13 ? f : idx,
  idx % 16 === 14 ? f : -idx,
  idx % 16 === 15 ? f : idx]]

  )
};

// Cases: f32_matDxD_[non_]const
const f32_cases = [2, 3, 4].
flatMap((dim) =>
[true, false].map((nonConst) => ({
  [`f32_mat${dim}x${dim}_${nonConst ? 'non_const' : 'const'}`]: () => {
    return FP.f32.generateMatrixToScalarCases(
      kDeterminantMatrixValues[dim],
      nonConst ? 'unfiltered' : 'finite',
      FP.f32.determinantInterval
    );
  }
}))
).
reduce((a, b) => ({ ...a, ...b }), {});

// Cases: f16_matDxD_[non_]const
const f16_cases = [2, 3, 4].
flatMap((dim) =>
[true, false].map((nonConst) => ({
  [`f16_mat${dim}x${dim}_${nonConst ? 'non_const' : 'const'}`]: () => {
    return FP.f16.generateMatrixToScalarCases(
      kDeterminantMatrixValues[dim],
      nonConst ? 'unfiltered' : 'finite',
      FP.f16.determinantInterval
    );
  }
}))
).
reduce((a, b) => ({ ...a, ...b }), {});

export const d = makeCaseCache('determinant', {
  ...f32_cases,
  ...f16_cases
});