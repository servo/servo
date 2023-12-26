/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { FP } from '../../../../util/floating_point.js';import { sparseMatrixF64Range } from '../../../../util/math.js';import { makeCaseCache } from '../case_cache.js';

// Cases: matCxR
const mat_cases = [2, 3, 4].
flatMap((cols) =>
[2, 3, 4].map((rows) => ({
  [`mat${cols}x${rows}`]: () => {
    return FP.abstract.generateMatrixPairToMatrixCases(
      sparseMatrixF64Range(cols, rows),
      sparseMatrixF64Range(cols, rows),
      'finite',
      FP.abstract.additionMatrixMatrixInterval
    );
  }
}))
).
reduce((a, b) => ({ ...a, ...b }), {});

export const d = makeCaseCache('binary/af_matrix_addition', mat_cases);