/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { FP } from '../../../../util/floating_point.js';import { sparseMatrixF64Range } from '../../../../util/math.js';import { selectNCases } from '../case.js';
import { makeCaseCache } from '../case_cache.js';

// Cases: matKxR_matCxK
const mat_mat_cases = [2, 3, 4].
flatMap((k) =>
[2, 3, 4].flatMap((cols) =>
[2, 3, 4].map((rows) => ({
  [`mat${k}x${rows}_mat${cols}x${k}`]: () => {
    return selectNCases(
      'binary/af_matrix_matrix_multiplication',
      10,
      FP.abstract.generateMatrixPairToMatrixCases(
        sparseMatrixF64Range(k, rows),
        sparseMatrixF64Range(cols, k),
        'finite',
        // Matrix-matrix multiplication has an inherited accuracy, so abstract is only expected to be as accurate as f32
        FP.f32.multiplicationMatrixMatrixInterval
      )
    );
  }
}))
)
).
reduce((a, b) => ({ ...a, ...b }), {});

export const d = makeCaseCache('binary/af_matrix_matrix_multiplication', mat_mat_cases);