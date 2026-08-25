/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { FP } from '../../../../util/floating_point.js';import { sparseMatrixF16Range, sparseScalarF16Range } from '../../../../util/math.js';import { makeCaseCache } from '../case_cache.js';

// Cases: matCxR_scalar_[non_]const
const mat_scalar_cases = [2, 3, 4].
flatMap((cols) =>
[2, 3, 4].flatMap((rows) =>
[true, false].map((nonConst) => ({
  [`mat${cols}x${rows}_scalar_${nonConst ? 'non_const' : 'const'}`]: () => {
    return FP.f16.generateMatrixScalarToMatrixCases(
      sparseMatrixF16Range(cols, rows),
      sparseScalarF16Range(),
      nonConst ? 'unfiltered' : 'finite',
      FP.f16.multiplicationMatrixScalarInterval
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
    return FP.f16.generateScalarMatrixToMatrixCases(
      sparseScalarF16Range(),
      sparseMatrixF16Range(cols, rows),
      nonConst ? 'unfiltered' : 'finite',
      FP.f16.multiplicationScalarMatrixInterval
    );
  }
}))
)
).
reduce((a, b) => ({ ...a, ...b }), {});

export const d = makeCaseCache('binary/f16_matrix_scalar_multiplication', {
  ...mat_scalar_cases,
  ...scalar_mat_cases
});