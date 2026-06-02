/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { FP } from '../../../../util/floating_point.js';import { sparseMatrixF32Range, sparseVectorF32Range } from '../../../../util/math.js';import { makeCaseCache } from '../case_cache.js';

// Cases: matCxR_vecC_[non_]const
const mat_vec_cases = [2, 3, 4].
flatMap((cols) =>
[2, 3, 4].flatMap((rows) =>
[true, false].map((nonConst) => ({
  [`mat${cols}x${rows}_vec${cols}_${nonConst ? 'non_const' : 'const'}`]: () => {
    return FP.f32.generateMatrixVectorToVectorCases(
      sparseMatrixF32Range(cols, rows),
      sparseVectorF32Range(cols),
      nonConst ? 'unfiltered' : 'finite',
      FP.f32.multiplicationMatrixVectorInterval
    );
  }
}))
)
).
reduce((a, b) => ({ ...a, ...b }), {});

// Cases: vecR_matCxR_[non_]const
const vec_mat_cases = [2, 3, 4].
flatMap((rows) =>
[2, 3, 4].flatMap((cols) =>
[true, false].map((nonConst) => ({
  [`vec${rows}_mat${cols}x${rows}_${nonConst ? 'non_const' : 'const'}`]: () => {
    return FP.f32.generateVectorMatrixToVectorCases(
      sparseVectorF32Range(rows),
      sparseMatrixF32Range(cols, rows),
      nonConst ? 'unfiltered' : 'finite',
      FP.f32.multiplicationVectorMatrixInterval
    );
  }
}))
)
).
reduce((a, b) => ({ ...a, ...b }), {});

export const d = makeCaseCache('binary/f32_matrix_vector_multiplication', {
  ...mat_vec_cases,
  ...vec_mat_cases
});