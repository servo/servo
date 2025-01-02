/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { assert } from '../../../../../../common/util/util.js';import { FP } from '../../../../../util/floating_point.js';import { PRNG } from '../../../../../util/prng.js';
import { makeCaseCache } from '../../case_cache.js';

// We want each case to avoid losing accuracy. We want to prevent bits from
// falling off the bottom of the significand during intermediate
// calculations. We want this to be true for reasonable implementations of
// the determinant. Let's compute a budget B for the number of significant
// bits allowed for each element in the matrix.
//
// Example computation method 1:  Use lower-diagonal form:
//  A matrix is in lower-diagonal form if all the entries above and to
//  the right of the diagonal are zero.
//  1. Convert the matrix to lower-diagonal form using elementary row
//  operations.
//  2. Then the determinant is the product of the elements on the diagonal.
//
//  Analysis:
//   - Let M be the number of significand bits for the floating point type.
//     Remember to include the hidden/implicit 1, because we will assume
//     all numbers are normal.
//   - It takes N-1 elementary row operations to convert a general NxN
//     matrix to lower diagonal form.  Let's ignore the fact this uses
//     division. Let's allocate a budget 1 bit of precision per row operation.
//   - If the diagonal elements are integers bounded by D, then their product
//     is bounded by D**N.  So we split the remaining significand bit budget
//     bits N ways.
//
//  So we have B ~ (M - (N-1)) / N
//
//  For f32, M = 24:
//     N = 4 :  B =~ ( 24 - 3 ) / 4 = 21 / 4
//              The bound on matrix elements should be 2**(21/4) or roughly 38.
//     N = 3 :  B =~ ( 24 - 2 ) / 3 = 22 / 3
//              The bound on matrix elements should be 2**(22/3) or roughly 161.
//     N = 2 :  B =~ ( 24 - 1 ) / 2 = 23 / 2
//              The bound on matrix elements should be 2**(23/2) or roughly 2896.
//
//  For f16, M = 11:
//     N = 4 :  B =~ ( 11 - 3 ) / 4 = 8 / 4 = 2
//              The bound on matrix elements should be 2**2 = 4.
//     N = 3 :  B =~ ( 11 - 2 ) / 3 = 9 / 3 = 3
//              The bound on matrix elements should be 2**3 = 8.
//     N = 2 :  B =~ ( 11 - 1 ) / 2 = 10 / 2 = 5
//              The bound on matrix elements should be 2**5 = 32.
//
// Example computation method 2: Naive method:
//   The naive calculation of a determinant of an NxN matrix is the alternating
//   sum of N terms, where each term is an entry from the matrix multiplied
//   by the determinant of an (N-1)x(N-1) submatrix.
//   The recurrence is:   T(N) = (N-1) + N + T(N-1), with T(1) = 0.
//   The first N is for the adds and subtracts, the second N is for the multiplies,
//   and the T(N-1) is the determinant of the submatrix.
//   As N grows, this blows up quickly, even when accounting for reusing results.
//   It's unlikely any GPU would actually calculate it this way.
//
// Accuracy for determinant is only defined for e, where e is an integer and
// |e| < quadroot(2**21) [~38],
// due to computational complexity of calculating the general solution for 4x4,
// so custom matrices are used.




// Number of random matrices to test, per configuration.
const numSamples = 20;

// Returns a random element in a range suitable for a square matrix
// that can have an accurately computed determinant.
function randomMatrixEntry(p, dim, fpwidth) {
  // See above for the justification.
  const rangeTable = {
    16: {
      2: 32,
      3: 8,
      4: 4
    },
    32: {
      2: 2896,
      3: 161,
      4: 38
    }
  };
  const N = rangeTable[fpwidth][dim];
  // Centre the distribution roughly at zero.
  const balanced = p.uniformInt(N) - Math.floor(N / 2);
  return balanced;
}

// Returns a random square matrix that should have an exactly computed
// determinant for the given floating point width.
// At least some of the matrices returned by this function should have
// a non-zero determinant. This will be checked later by the nonTrivialMatrices
// function.
function randomSquareMatrix(p, dim, fpwidth) {
  const result = [...Array(dim)].map((_) => [...Array(dim)]);
  // Scale each element by a simple power of two. This should only affect
  // the exponent of the result.
  const multiplier = [1, 2, 0.25][p.uniformInt(3)];
  for (let c = 0; c < dim; c++) {
    for (let r = 0; r < dim; r++) {
      result[c][r] = multiplier * randomMatrixEntry(p, dim, fpwidth);
    }
  }
  return result;
}



// Returns true if at least one of the matrices has a non-zero determinant.
function nonTrivialMatrices(matrices, detFn) {
  const detInterval = (m) => detFn(m);
  const sumBegin = matrices.reduce((accum, m) => accum + Math.abs(detInterval(m).begin), 0);
  const sumEnd = matrices.reduce((accum, m) => accum + Math.abs(detInterval(m).end), 0);
  return sumBegin > 0 && sumEnd >= sumBegin;
}

// Cases: f32_matDxD_[non_]const
const f32_cases = [2, 3, 4].
flatMap((dim) =>
[true, false].map((nonConst) => ({
  [`f32_mat${dim}x${dim}_${nonConst ? 'non_const' : 'const'}`]: () => {
    const p = new PRNG(dim + 32);
    const matrices = [...Array(numSamples)].map((_) =>
    randomSquareMatrix(p, dim, 32)
    );
    assert(nonTrivialMatrices(matrices, FP.f32.determinantInterval));
    return FP.f32.generateMatrixToScalarCases(
      matrices,
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
    const p = new PRNG(dim + 16);
    const matrices = [...Array(numSamples)].map((_) =>
    randomSquareMatrix(p, dim, 16)
    );
    assert(nonTrivialMatrices(matrices, FP.f16.determinantInterval));
    return FP.f16.generateMatrixToScalarCases(
      matrices,
      nonConst ? 'unfiltered' : 'finite',
      FP.f16.determinantInterval
    );
  }
}))
).
reduce((a, b) => ({ ...a, ...b }), {});

// Cases: abstract_matDxD
const abstract_cases = [2, 3, 4].
map((dim) => ({
  [`abstract_mat${dim}x${dim}`]: () => {
    const p = new PRNG(dim + 64);
    // Use f32 values range for abstract float.
    const matrices = [...Array(numSamples)].map((_) =>
    randomSquareMatrix(p, dim, 32)
    );
    assert(nonTrivialMatrices(matrices, FP.f32.determinantInterval));
    return FP.abstract.generateMatrixToScalarCases(
      matrices,
      'finite',
      // determinant has an inherited accuracy, so abstract is only expected to be as accurate as f32
      FP.f32.determinantInterval
    );
  }
})).
reduce((a, b) => ({ ...a, ...b }), {});

export const d = makeCaseCache('determinant', {
  ...f32_cases,
  ...f16_cases,
  ...abstract_cases
});