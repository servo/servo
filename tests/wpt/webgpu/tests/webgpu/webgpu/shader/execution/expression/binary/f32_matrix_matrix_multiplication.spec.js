/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Execution Tests for matrix-matrix f32 multiplication expression
`;
import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../gpu_test.js';
import { TypeF32, TypeMat } from '../../../../util/conversion.js';
import { FP } from '../../../../util/floating_point.js';
import { sparseMatrixF32Range } from '../../../../util/math.js';
import { makeCaseCache } from '../case_cache.js';
import { allInputSources, run } from '../expression.js';

import { binary, compoundBinary } from './binary.js';

export const g = makeTestGroup(GPUTest);

export const d = makeCaseCache('binary/f32_matrix_matrix_multiplication', {
  mat2x2_mat2x2_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(2, 2),
      sparseMatrixF32Range(2, 2),
      'finite',
      FP.f32.multiplicationMatrixMatrixInterval
    );
  },
  mat2x2_mat2x2_non_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(2, 2),
      sparseMatrixF32Range(2, 2),
      'unfiltered',
      FP.f32.multiplicationMatrixMatrixInterval
    );
  },
  mat2x3_mat2x2_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(2, 3),
      sparseMatrixF32Range(2, 2),
      'finite',
      FP.f32.multiplicationMatrixMatrixInterval
    );
  },
  mat2x3_mat2x2_non_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(2, 3),
      sparseMatrixF32Range(2, 2),
      'unfiltered',
      FP.f32.multiplicationMatrixMatrixInterval
    );
  },
  mat2x2_mat3x2_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(2, 2),
      sparseMatrixF32Range(3, 2),
      'finite',
      FP.f32.multiplicationMatrixMatrixInterval
    );
  },
  mat2x2_mat3x2_non_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(2, 2),
      sparseMatrixF32Range(3, 2),
      'unfiltered',
      FP.f32.multiplicationMatrixMatrixInterval
    );
  },
  mat2x3_mat3x2_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(2, 3),
      sparseMatrixF32Range(3, 2),
      'finite',
      FP.f32.multiplicationMatrixMatrixInterval
    );
  },
  mat2x3_mat3x2_non_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(2, 3),
      sparseMatrixF32Range(3, 2),
      'unfiltered',
      FP.f32.multiplicationMatrixMatrixInterval
    );
  },
  mat2x4_mat2x2_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(2, 4),
      sparseMatrixF32Range(2, 2),
      'finite',
      FP.f32.multiplicationMatrixMatrixInterval
    );
  },
  mat2x4_mat2x2_non_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(2, 4),
      sparseMatrixF32Range(2, 2),
      'unfiltered',
      FP.f32.multiplicationMatrixMatrixInterval
    );
  },
  mat2x2_mat4x2_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(2, 2),
      sparseMatrixF32Range(4, 2),
      'finite',
      FP.f32.multiplicationMatrixMatrixInterval
    );
  },
  mat2x2_mat4x2_non_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(2, 2),
      sparseMatrixF32Range(4, 2),
      'unfiltered',
      FP.f32.multiplicationMatrixMatrixInterval
    );
  },
  mat2x4_mat4x2_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(2, 4),
      sparseMatrixF32Range(4, 2),
      'finite',
      FP.f32.multiplicationMatrixMatrixInterval
    );
  },
  mat2x4_mat4x2_non_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(2, 4),
      sparseMatrixF32Range(4, 2),
      'unfiltered',
      FP.f32.multiplicationMatrixMatrixInterval
    );
  },
  mat2x3_mat4x2_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(2, 3),
      sparseMatrixF32Range(4, 2),
      'finite',
      FP.f32.multiplicationMatrixMatrixInterval
    );
  },
  mat2x3_mat4x2_non_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(2, 3),
      sparseMatrixF32Range(4, 2),
      'unfiltered',
      FP.f32.multiplicationMatrixMatrixInterval
    );
  },
  mat2x4_mat3x2_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(2, 4),
      sparseMatrixF32Range(3, 2),
      'finite',
      FP.f32.multiplicationMatrixMatrixInterval
    );
  },
  mat2x4_mat3x2_non_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(2, 4),
      sparseMatrixF32Range(3, 2),
      'unfiltered',
      FP.f32.multiplicationMatrixMatrixInterval
    );
  },
  mat3x3_mat3x3_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(3, 3),
      sparseMatrixF32Range(3, 3),
      'finite',
      FP.f32.multiplicationMatrixMatrixInterval
    );
  },
  mat3x3_mat3x3_non_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(3, 3),
      sparseMatrixF32Range(3, 3),
      'unfiltered',
      FP.f32.multiplicationMatrixMatrixInterval
    );
  },
  mat3x2_mat3x3_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(3, 2),
      sparseMatrixF32Range(3, 3),
      'finite',
      FP.f32.multiplicationMatrixMatrixInterval
    );
  },
  mat3x2_mat3x3_non_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(3, 2),
      sparseMatrixF32Range(3, 3),
      'unfiltered',
      FP.f32.multiplicationMatrixMatrixInterval
    );
  },
  mat3x3_mat2x3_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(3, 3),
      sparseMatrixF32Range(2, 3),
      'finite',
      FP.f32.multiplicationMatrixMatrixInterval
    );
  },
  mat3x3_mat2x3_non_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(3, 3),
      sparseMatrixF32Range(2, 3),
      'unfiltered',
      FP.f32.multiplicationMatrixMatrixInterval
    );
  },
  mat3x2_mat2x3_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(3, 2),
      sparseMatrixF32Range(2, 3),
      'finite',
      FP.f32.multiplicationMatrixMatrixInterval
    );
  },
  mat3x2_mat2x3_non_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(3, 2),
      sparseMatrixF32Range(2, 3),
      'unfiltered',
      FP.f32.multiplicationMatrixMatrixInterval
    );
  },
  mat3x4_mat3x3_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(3, 4),
      sparseMatrixF32Range(3, 3),
      'finite',
      FP.f32.multiplicationMatrixMatrixInterval
    );
  },
  mat3x4_mat3x3_non_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(3, 4),
      sparseMatrixF32Range(3, 3),
      'unfiltered',
      FP.f32.multiplicationMatrixMatrixInterval
    );
  },
  mat3x3_mat4x3_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(3, 3),
      sparseMatrixF32Range(4, 3),
      'finite',
      FP.f32.multiplicationMatrixMatrixInterval
    );
  },
  mat3x3_mat4x3_non_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(3, 3),
      sparseMatrixF32Range(4, 3),
      'unfiltered',
      FP.f32.multiplicationMatrixMatrixInterval
    );
  },
  mat3x4_mat4x3_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(3, 4),
      sparseMatrixF32Range(4, 3),
      'finite',
      FP.f32.multiplicationMatrixMatrixInterval
    );
  },
  mat3x4_mat4x3_non_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(3, 4),
      sparseMatrixF32Range(4, 3),
      'unfiltered',
      FP.f32.multiplicationMatrixMatrixInterval
    );
  },
  mat3x2_mat4x3_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(3, 2),
      sparseMatrixF32Range(4, 3),
      'finite',
      FP.f32.multiplicationMatrixMatrixInterval
    );
  },
  mat3x2_mat4x3_non_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(3, 2),
      sparseMatrixF32Range(4, 3),
      'unfiltered',
      FP.f32.multiplicationMatrixMatrixInterval
    );
  },
  mat3x4_mat2x3_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(3, 4),
      sparseMatrixF32Range(2, 3),
      'finite',
      FP.f32.multiplicationMatrixMatrixInterval
    );
  },
  mat3x4_mat2x3_non_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(3, 4),
      sparseMatrixF32Range(2, 3),
      'unfiltered',
      FP.f32.multiplicationMatrixMatrixInterval
    );
  },
  mat4x4_mat4x4_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(4, 4),
      sparseMatrixF32Range(4, 4),
      'finite',
      FP.f32.multiplicationMatrixMatrixInterval
    );
  },
  mat4x4_mat4x4_non_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(4, 4),
      sparseMatrixF32Range(4, 4),
      'unfiltered',
      FP.f32.multiplicationMatrixMatrixInterval
    );
  },
  mat4x2_mat4x4_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(4, 2),
      sparseMatrixF32Range(4, 4),
      'finite',
      FP.f32.multiplicationMatrixMatrixInterval
    );
  },
  mat4x2_mat4x4_non_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(4, 2),
      sparseMatrixF32Range(4, 4),
      'unfiltered',
      FP.f32.multiplicationMatrixMatrixInterval
    );
  },
  mat4x4_mat2x4_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(4, 4),
      sparseMatrixF32Range(2, 4),
      'finite',
      FP.f32.multiplicationMatrixMatrixInterval
    );
  },
  mat4x4_mat2x4_non_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(4, 4),
      sparseMatrixF32Range(2, 4),
      'unfiltered',
      FP.f32.multiplicationMatrixMatrixInterval
    );
  },
  mat4x2_mat2x4_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(4, 2),
      sparseMatrixF32Range(2, 4),
      'finite',
      FP.f32.multiplicationMatrixMatrixInterval
    );
  },
  mat4x2_mat2x4_non_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(4, 2),
      sparseMatrixF32Range(2, 4),
      'unfiltered',
      FP.f32.multiplicationMatrixMatrixInterval
    );
  },
  mat4x3_mat4x4_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(4, 3),
      sparseMatrixF32Range(4, 4),
      'finite',
      FP.f32.multiplicationMatrixMatrixInterval
    );
  },
  mat4x3_mat4x4_non_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(4, 3),
      sparseMatrixF32Range(4, 4),
      'unfiltered',
      FP.f32.multiplicationMatrixMatrixInterval
    );
  },
  mat4x4_mat3x4_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(4, 4),
      sparseMatrixF32Range(3, 4),
      'finite',
      FP.f32.multiplicationMatrixMatrixInterval
    );
  },
  mat4x4_mat3x4_non_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(4, 4),
      sparseMatrixF32Range(3, 4),
      'unfiltered',
      FP.f32.multiplicationMatrixMatrixInterval
    );
  },
  mat4x3_mat3x4_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(4, 3),
      sparseMatrixF32Range(3, 4),
      'finite',
      FP.f32.multiplicationMatrixMatrixInterval
    );
  },
  mat4x3_mat3x4_non_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(4, 3),
      sparseMatrixF32Range(3, 4),
      'unfiltered',
      FP.f32.multiplicationMatrixMatrixInterval
    );
  },
  mat4x2_mat3x4_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(4, 2),
      sparseMatrixF32Range(3, 4),
      'finite',
      FP.f32.multiplicationMatrixMatrixInterval
    );
  },
  mat4x2_mat3x4_non_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(4, 2),
      sparseMatrixF32Range(3, 4),
      'unfiltered',
      FP.f32.multiplicationMatrixMatrixInterval
    );
  },
  mat4x3_mat2x4_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(4, 3),
      sparseMatrixF32Range(2, 4),
      'finite',
      FP.f32.multiplicationMatrixMatrixInterval
    );
  },
  mat4x3_mat2x4_non_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(4, 3),
      sparseMatrixF32Range(2, 4),
      'unfiltered',
      FP.f32.multiplicationMatrixMatrixInterval
    );
  },
});

g.test('matrix_matrix')
  .specURL('https://www.w3.org/TR/WGSL/#floating-point-evaluation')
  .desc(
    `
Expression: x * y, where x is a matrix and y is a matrix
Accuracy: Correctly rounded
`
  )
  .params(u =>
    u
      .combine('inputSource', allInputSources)
      .combine('common_dim', [2, 3, 4])
      .combine('x_rows', [2, 3, 4])
      .combine('y_cols', [2, 3, 4])
  )
  .fn(async t => {
    const x_cols = t.params.common_dim;
    const x_rows = t.params.x_rows;
    const y_cols = t.params.y_cols;
    const y_rows = t.params.common_dim;

    const cases = await d.get(
      t.params.inputSource === 'const'
        ? `mat${x_cols}x${x_rows}_mat${y_cols}x${y_rows}_const`
        : `mat${x_cols}x${x_rows}_mat${y_cols}x${y_rows}_non_const`
    );

    await run(
      t,
      binary('*'),
      [TypeMat(x_cols, x_rows, TypeF32), TypeMat(y_cols, y_rows, TypeF32)],
      TypeMat(y_cols, x_rows, TypeF32),
      t.params,
      cases
    );
  });

g.test('matrix_matrix_compound')
  .specURL('https://www.w3.org/TR/WGSL/#floating-point-evaluation')
  .desc(
    `
Expression: x *= y, where x is a matrix and y is a matrix
Accuracy: Correctly rounded
`
  )
  .params(u =>
    u
      .combine('inputSource', allInputSources)
      .combine('common_dim', [2, 3, 4])
      .combine('x_rows', [2, 3, 4])
  )
  .fn(async t => {
    const x_cols = t.params.common_dim;
    const x_rows = t.params.x_rows;
    const y_cols = x_cols;
    const y_rows = t.params.common_dim;

    const cases = await d.get(
      t.params.inputSource === 'const'
        ? `mat${x_cols}x${x_rows}_mat${y_cols}x${y_rows}_const`
        : `mat${x_cols}x${x_rows}_mat${y_cols}x${y_rows}_non_const`
    );

    await run(
      t,
      compoundBinary('*='),
      [TypeMat(x_cols, x_rows, TypeF32), TypeMat(y_cols, y_rows, TypeF32)],
      TypeMat(y_cols, x_rows, TypeF32),
      t.params,
      cases
    );
  });
