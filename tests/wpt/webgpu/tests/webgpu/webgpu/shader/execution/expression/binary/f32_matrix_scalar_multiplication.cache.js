/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { FP } from '../../../../util/floating_point.js';import { sparseMatrixF32Range, sparseScalarF32Range } from '../../../../util/math.js';import { makeCaseCache } from '../case_cache.js';

// Cases: matCxR_scalar_[non_]const
const mat_scalar_cases = [2, 3, 4].
flatMap((cols) =>
[2, 3, 4].flatMap((rows) =>
[true, false].map((nonConst) => ({
  [`mat${cols}x${rows}_scalar_${nonConst ? 'non_const' : 'const'}`]: () => {
    return FP.f32.generateMatrixScalarToMatrixCases(
      sparseMatrixF32Range(cols, rows),
      sparseScalarF32Range(),
      nonConst ? 'unfiltered' : 'finite',
      FP.f32.multiplicationMatrixScalarInterval
    );
  }
}))
)
).
reduce((a, b) => ({ ...a, ...b }), {});

// Cases: scalar_matCxR_[non_]const
const scalar_mat_cases = [2, 3, 4].
flatMap((cols) =>
[2, 3, 4].flatMap((rows) =>
[true, false].map((nonConst) => ({
  [`scalar_mat${cols}x${rows}_${nonConst ? 'non_const' : 'const'}`]: () => {
    return FP.f32.generateScalarMatrixToMatrixCases(
      sparseScalarF32Range(),
      sparseMatrixF32Range(cols, rows),
      nonConst ? 'unfiltered' : 'finite',
      FP.f32.multiplicationScalarMatrixInterval
    );
  }
}))
)
).
reduce((a, b) => ({ ...a, ...b }), {});

export const d = makeCaseCache('binary/f32_matrix_scalar_multiplication', {
  ...mat_scalar_cases,
  ...scalar_mat_cases
});