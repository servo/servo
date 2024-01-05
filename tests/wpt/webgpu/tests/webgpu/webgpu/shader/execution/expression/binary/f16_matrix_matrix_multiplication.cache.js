/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { FP } from '../../../../util/floating_point.js';import { sparseMatrixF16Range } from '../../../../util/math.js';import { makeCaseCache } from '../case_cache.js';

// Cases: matKxR_matCxK_[non_]const
const mat_mat_cases = [2, 3, 4].
flatMap((k) =>
[2, 3, 4].flatMap((cols) =>
[2, 3, 4].flatMap((rows) =>
[true, false].map((nonConst) => ({
  [`mat${k}x${rows}_mat${cols}x${k}_${nonConst ? 'non_const' : 'const'}`]: () => {
    return FP.f16.generateMatrixPairToMatrixCases(
      sparseMatrixF16Range(k, rows),
      sparseMatrixF16Range(cols, k),
      nonConst ? 'unfiltered' : 'finite',
      FP.f16.multiplicationMatrixMatrixInterval
    );
  }
}))
)
)
).
reduce((a, b) => ({ ...a, ...b }), {});

export const d = makeCaseCache('binary/f16_matrix_matrix_multiplication', mat_mat_cases);