/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Execution Tests for matrix-scalar and scalar-matrix f32 multiplication expression
`;
import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../gpu_test.js';
import { TypeF32, TypeMat } from '../../../../util/conversion.js';
import { FP } from '../../../../util/floating_point.js';
import { sparseF32Range, sparseMatrixF32Range } from '../../../../util/math.js';
import { makeCaseCache } from '../case_cache.js';
import { allInputSources, run } from '../expression.js';

import { binary, compoundBinary } from './binary.js';

export const g = makeTestGroup(GPUTest);

export const d = makeCaseCache('binary/f32_matrix_scalar_multiplication', {
  mat2x2_scalar_const: () => {
    return FP.f32.generateMatrixScalarToMatrixCases(
      sparseMatrixF32Range(2, 2),
      sparseF32Range(),
      'finite',
      FP.f32.multiplicationMatrixScalarInterval
    );
  },
  mat2x2_scalar_non_const: () => {
    return FP.f32.generateMatrixScalarToMatrixCases(
      sparseMatrixF32Range(2, 2),
      sparseF32Range(),
      'unfiltered',
      FP.f32.multiplicationMatrixScalarInterval
    );
  },
  mat2x3_scalar_const: () => {
    return FP.f32.generateMatrixScalarToMatrixCases(
      sparseMatrixF32Range(2, 3),
      sparseF32Range(),
      'finite',
      FP.f32.multiplicationMatrixScalarInterval
    );
  },
  mat2x3_scalar_non_const: () => {
    return FP.f32.generateMatrixScalarToMatrixCases(
      sparseMatrixF32Range(2, 3),
      sparseF32Range(),
      'unfiltered',
      FP.f32.multiplicationMatrixScalarInterval
    );
  },
  mat2x4_scalar_const: () => {
    return FP.f32.generateMatrixScalarToMatrixCases(
      sparseMatrixF32Range(2, 4),
      sparseF32Range(),
      'finite',
      FP.f32.multiplicationMatrixScalarInterval
    );
  },
  mat2x4_scalar_non_const: () => {
    return FP.f32.generateMatrixScalarToMatrixCases(
      sparseMatrixF32Range(2, 4),
      sparseF32Range(),
      'unfiltered',
      FP.f32.multiplicationMatrixScalarInterval
    );
  },
  mat3x2_scalar_const: () => {
    return FP.f32.generateMatrixScalarToMatrixCases(
      sparseMatrixF32Range(3, 2),
      sparseF32Range(),
      'finite',
      FP.f32.multiplicationMatrixScalarInterval
    );
  },
  mat3x2_scalar_non_const: () => {
    return FP.f32.generateMatrixScalarToMatrixCases(
      sparseMatrixF32Range(3, 2),
      sparseF32Range(),
      'unfiltered',
      FP.f32.multiplicationMatrixScalarInterval
    );
  },
  mat3x3_scalar_const: () => {
    return FP.f32.generateMatrixScalarToMatrixCases(
      sparseMatrixF32Range(3, 3),
      sparseF32Range(),
      'finite',
      FP.f32.multiplicationMatrixScalarInterval
    );
  },
  mat3x3_scalar_non_const: () => {
    return FP.f32.generateMatrixScalarToMatrixCases(
      sparseMatrixF32Range(3, 3),
      sparseF32Range(),
      'unfiltered',
      FP.f32.multiplicationMatrixScalarInterval
    );
  },
  mat3x4_scalar_const: () => {
    return FP.f32.generateMatrixScalarToMatrixCases(
      sparseMatrixF32Range(3, 4),
      sparseF32Range(),
      'finite',
      FP.f32.multiplicationMatrixScalarInterval
    );
  },
  mat3x4_scalar_non_const: () => {
    return FP.f32.generateMatrixScalarToMatrixCases(
      sparseMatrixF32Range(3, 4),
      sparseF32Range(),
      'unfiltered',
      FP.f32.multiplicationMatrixScalarInterval
    );
  },
  mat4x2_scalar_const: () => {
    return FP.f32.generateMatrixScalarToMatrixCases(
      sparseMatrixF32Range(4, 2),
      sparseF32Range(),
      'finite',
      FP.f32.multiplicationMatrixScalarInterval
    );
  },
  mat4x2_scalar_non_const: () => {
    return FP.f32.generateMatrixScalarToMatrixCases(
      sparseMatrixF32Range(4, 2),
      sparseF32Range(),
      'unfiltered',
      FP.f32.multiplicationMatrixScalarInterval
    );
  },
  mat4x3_scalar_const: () => {
    return FP.f32.generateMatrixScalarToMatrixCases(
      sparseMatrixF32Range(4, 3),
      sparseF32Range(),
      'finite',
      FP.f32.multiplicationMatrixScalarInterval
    );
  },
  mat4x3_scalar_non_const: () => {
    return FP.f32.generateMatrixScalarToMatrixCases(
      sparseMatrixF32Range(4, 3),
      sparseF32Range(),
      'unfiltered',
      FP.f32.multiplicationMatrixScalarInterval
    );
  },
  mat4x4_scalar_const: () => {
    return FP.f32.generateMatrixScalarToMatrixCases(
      sparseMatrixF32Range(4, 4),
      sparseF32Range(),
      'finite',
      FP.f32.multiplicationMatrixScalarInterval
    );
  },
  mat4x4_scalar_non_const: () => {
    return FP.f32.generateMatrixScalarToMatrixCases(
      sparseMatrixF32Range(4, 4),
      sparseF32Range(),
      'unfiltered',
      FP.f32.multiplicationMatrixScalarInterval
    );
  },
  scalar_mat2x2_const: () => {
    return FP.f32.generateScalarMatrixToMatrixCases(
      sparseF32Range(),
      sparseMatrixF32Range(2, 2),
      'finite',
      FP.f32.multiplicationScalarMatrixInterval
    );
  },
  scalar_mat2x2_non_const: () => {
    return FP.f32.generateScalarMatrixToMatrixCases(
      sparseF32Range(),
      sparseMatrixF32Range(2, 2),
      'unfiltered',
      FP.f32.multiplicationScalarMatrixInterval
    );
  },
  scalar_mat2x3_const: () => {
    return FP.f32.generateScalarMatrixToMatrixCases(
      sparseF32Range(),
      sparseMatrixF32Range(2, 3),
      'finite',
      FP.f32.multiplicationScalarMatrixInterval
    );
  },
  scalar_mat2x3_non_const: () => {
    return FP.f32.generateScalarMatrixToMatrixCases(
      sparseF32Range(),
      sparseMatrixF32Range(2, 3),
      'unfiltered',
      FP.f32.multiplicationScalarMatrixInterval
    );
  },
  scalar_mat2x4_const: () => {
    return FP.f32.generateScalarMatrixToMatrixCases(
      sparseF32Range(),
      sparseMatrixF32Range(2, 4),
      'finite',
      FP.f32.multiplicationScalarMatrixInterval
    );
  },
  scalar_mat2x4_non_const: () => {
    return FP.f32.generateScalarMatrixToMatrixCases(
      sparseF32Range(),
      sparseMatrixF32Range(2, 4),
      'unfiltered',
      FP.f32.multiplicationScalarMatrixInterval
    );
  },
  scalar_mat3x2_const: () => {
    return FP.f32.generateScalarMatrixToMatrixCases(
      sparseF32Range(),
      sparseMatrixF32Range(3, 2),
      'finite',
      FP.f32.multiplicationScalarMatrixInterval
    );
  },
  scalar_mat3x2_non_const: () => {
    return FP.f32.generateScalarMatrixToMatrixCases(
      sparseF32Range(),
      sparseMatrixF32Range(3, 2),
      'unfiltered',
      FP.f32.multiplicationScalarMatrixInterval
    );
  },
  scalar_mat3x3_const: () => {
    return FP.f32.generateScalarMatrixToMatrixCases(
      sparseF32Range(),
      sparseMatrixF32Range(3, 3),
      'finite',
      FP.f32.multiplicationScalarMatrixInterval
    );
  },
  scalar_mat3x3_non_const: () => {
    return FP.f32.generateScalarMatrixToMatrixCases(
      sparseF32Range(),
      sparseMatrixF32Range(3, 3),
      'unfiltered',
      FP.f32.multiplicationScalarMatrixInterval
    );
  },
  scalar_mat3x4_const: () => {
    return FP.f32.generateScalarMatrixToMatrixCases(
      sparseF32Range(),
      sparseMatrixF32Range(3, 4),
      'finite',
      FP.f32.multiplicationScalarMatrixInterval
    );
  },
  scalar_mat3x4_non_const: () => {
    return FP.f32.generateScalarMatrixToMatrixCases(
      sparseF32Range(),
      sparseMatrixF32Range(3, 4),
      'unfiltered',
      FP.f32.multiplicationScalarMatrixInterval
    );
  },
  scalar_mat4x2_const: () => {
    return FP.f32.generateScalarMatrixToMatrixCases(
      sparseF32Range(),
      sparseMatrixF32Range(4, 2),
      'finite',
      FP.f32.multiplicationScalarMatrixInterval
    );
  },
  scalar_mat4x2_non_const: () => {
    return FP.f32.generateScalarMatrixToMatrixCases(
      sparseF32Range(),
      sparseMatrixF32Range(4, 2),
      'unfiltered',
      FP.f32.multiplicationScalarMatrixInterval
    );
  },
  scalar_mat4x3_const: () => {
    return FP.f32.generateScalarMatrixToMatrixCases(
      sparseF32Range(),
      sparseMatrixF32Range(4, 3),
      'finite',
      FP.f32.multiplicationScalarMatrixInterval
    );
  },
  scalar_mat4x3_non_const: () => {
    return FP.f32.generateScalarMatrixToMatrixCases(
      sparseF32Range(),
      sparseMatrixF32Range(4, 3),
      'unfiltered',
      FP.f32.multiplicationScalarMatrixInterval
    );
  },
  scalar_mat4x4_const: () => {
    return FP.f32.generateScalarMatrixToMatrixCases(
      sparseF32Range(),
      sparseMatrixF32Range(4, 4),
      'finite',
      FP.f32.multiplicationScalarMatrixInterval
    );
  },
  scalar_mat4x4_non_const: () => {
    return FP.f32.generateScalarMatrixToMatrixCases(
      sparseF32Range(),
      sparseMatrixF32Range(4, 4),
      'unfiltered',
      FP.f32.multiplicationScalarMatrixInterval
    );
  },
});

g.test('matrix_scalar')
  .specURL('https://www.w3.org/TR/WGSL/#floating-point-evaluation')
  .desc(
    `
Expression: x * y, where x is a matrix and y is a scalar
Accuracy: Correctly rounded
`
  )
  .params(u =>
    u.combine('inputSource', allInputSources).combine('cols', [2, 3, 4]).combine('rows', [2, 3, 4])
  )
  .fn(async t => {
    const cols = t.params.cols;
    const rows = t.params.rows;
    const cases = await d.get(
      t.params.inputSource === 'const'
        ? `mat${cols}x${rows}_scalar_const`
        : `mat${cols}x${rows}_scalar_non_const`
    );

    await run(
      t,
      binary('*'),
      [TypeMat(cols, rows, TypeF32), TypeF32],
      TypeMat(cols, rows, TypeF32),
      t.params,
      cases
    );
  });

g.test('matrix_scalar_compound')
  .specURL('https://www.w3.org/TR/WGSL/#floating-point-evaluation')
  .desc(
    `
Expression: x *= y, where x is a matrix and y is a scalar
Accuracy: Correctly rounded
`
  )
  .params(u =>
    u.combine('inputSource', allInputSources).combine('cols', [2, 3, 4]).combine('rows', [2, 3, 4])
  )
  .fn(async t => {
    const cols = t.params.cols;
    const rows = t.params.rows;
    const cases = await d.get(
      t.params.inputSource === 'const'
        ? `mat${cols}x${rows}_scalar_const`
        : `mat${cols}x${rows}_scalar_non_const`
    );

    await run(
      t,
      compoundBinary('*='),
      [TypeMat(cols, rows, TypeF32), TypeF32],
      TypeMat(cols, rows, TypeF32),
      t.params,
      cases
    );
  });

g.test('scalar_matrix')
  .specURL('https://www.w3.org/TR/WGSL/#floating-point-evaluation')
  .desc(
    `
Expression: x * y, where x is a scalar and y is a matrix
Accuracy: Correctly rounded
`
  )
  .params(u =>
    u.combine('inputSource', allInputSources).combine('cols', [2, 3, 4]).combine('rows', [2, 3, 4])
  )
  .fn(async t => {
    const cols = t.params.cols;
    const rows = t.params.rows;
    const cases = await d.get(
      t.params.inputSource === 'const'
        ? `scalar_mat${cols}x${rows}_const`
        : `scalar_mat${cols}x${rows}_non_const`
    );

    await run(
      t,
      binary('*'),
      [TypeF32, TypeMat(cols, rows, TypeF32)],
      TypeMat(cols, rows, TypeF32),
      t.params,
      cases
    );
  });
