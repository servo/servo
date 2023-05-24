/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Execution Tests for matrix-vector and vector-matrix f32 multiplication expression
`;
import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../gpu_test.js';
import { TypeF32, TypeMat, TypeVec } from '../../../../util/conversion.js';
import { FP } from '../../../../util/floating_point.js';
import { sparseMatrixF32Range, sparseVectorF32Range } from '../../../../util/math.js';
import { makeCaseCache } from '../case_cache.js';
import { allInputSources, run } from '../expression.js';

import { binary, compoundBinary } from './binary.js';

export const g = makeTestGroup(GPUTest);

export const d = makeCaseCache('binary/f32_matrix_vector_multiplication', {
  mat2x2_vec2_const: () => {
    return FP.f32.generateMatrixVectorToVectorCases(
      sparseMatrixF32Range(2, 2),
      sparseVectorF32Range(2),
      'finite',
      FP.f32.multiplicationMatrixVectorInterval
    );
  },
  mat2x2_vec2_non_const: () => {
    return FP.f32.generateMatrixVectorToVectorCases(
      sparseMatrixF32Range(2, 2),
      sparseVectorF32Range(2),
      'unfiltered',
      FP.f32.multiplicationMatrixVectorInterval
    );
  },
  mat2x3_vec2_const: () => {
    return FP.f32.generateMatrixVectorToVectorCases(
      sparseMatrixF32Range(2, 3),
      sparseVectorF32Range(2),
      'finite',
      FP.f32.multiplicationMatrixVectorInterval
    );
  },
  mat2x3_vec2_non_const: () => {
    return FP.f32.generateMatrixVectorToVectorCases(
      sparseMatrixF32Range(2, 3),
      sparseVectorF32Range(2),
      'unfiltered',
      FP.f32.multiplicationMatrixVectorInterval
    );
  },
  mat2x4_vec2_const: () => {
    return FP.f32.generateMatrixVectorToVectorCases(
      sparseMatrixF32Range(2, 4),
      sparseVectorF32Range(2),
      'finite',
      FP.f32.multiplicationMatrixVectorInterval
    );
  },
  mat2x4_vec2_non_const: () => {
    return FP.f32.generateMatrixVectorToVectorCases(
      sparseMatrixF32Range(2, 4),
      sparseVectorF32Range(2),
      'unfiltered',
      FP.f32.multiplicationMatrixVectorInterval
    );
  },
  mat3x2_vec3_const: () => {
    return FP.f32.generateMatrixVectorToVectorCases(
      sparseMatrixF32Range(3, 2),
      sparseVectorF32Range(3),
      'finite',
      FP.f32.multiplicationMatrixVectorInterval
    );
  },
  mat3x2_vec3_non_const: () => {
    return FP.f32.generateMatrixVectorToVectorCases(
      sparseMatrixF32Range(3, 2),
      sparseVectorF32Range(3),
      'unfiltered',
      FP.f32.multiplicationMatrixVectorInterval
    );
  },
  mat3x3_vec3_const: () => {
    return FP.f32.generateMatrixVectorToVectorCases(
      sparseMatrixF32Range(3, 3),
      sparseVectorF32Range(3),
      'finite',
      FP.f32.multiplicationMatrixVectorInterval
    );
  },
  mat3x3_vec3_non_const: () => {
    return FP.f32.generateMatrixVectorToVectorCases(
      sparseMatrixF32Range(3, 3),
      sparseVectorF32Range(3),
      'unfiltered',
      FP.f32.multiplicationMatrixVectorInterval
    );
  },
  mat3x4_vec3_const: () => {
    return FP.f32.generateMatrixVectorToVectorCases(
      sparseMatrixF32Range(3, 4),
      sparseVectorF32Range(3),
      'finite',
      FP.f32.multiplicationMatrixVectorInterval
    );
  },
  mat3x4_vec3_non_const: () => {
    return FP.f32.generateMatrixVectorToVectorCases(
      sparseMatrixF32Range(3, 4),
      sparseVectorF32Range(3),
      'unfiltered',
      FP.f32.multiplicationMatrixVectorInterval
    );
  },
  mat4x2_vec4_const: () => {
    return FP.f32.generateMatrixVectorToVectorCases(
      sparseMatrixF32Range(4, 2),
      sparseVectorF32Range(4),
      'finite',
      FP.f32.multiplicationMatrixVectorInterval
    );
  },
  mat4x2_vec4_non_const: () => {
    return FP.f32.generateMatrixVectorToVectorCases(
      sparseMatrixF32Range(4, 2),
      sparseVectorF32Range(4),
      'unfiltered',
      FP.f32.multiplicationMatrixVectorInterval
    );
  },
  mat4x3_vec4_const: () => {
    return FP.f32.generateMatrixVectorToVectorCases(
      sparseMatrixF32Range(4, 3),
      sparseVectorF32Range(4),
      'finite',
      FP.f32.multiplicationMatrixVectorInterval
    );
  },
  mat4x3_vec4_non_const: () => {
    return FP.f32.generateMatrixVectorToVectorCases(
      sparseMatrixF32Range(4, 3),
      sparseVectorF32Range(4),
      'unfiltered',
      FP.f32.multiplicationMatrixVectorInterval
    );
  },
  mat4x4_vec4_const: () => {
    return FP.f32.generateMatrixVectorToVectorCases(
      sparseMatrixF32Range(4, 4),
      sparseVectorF32Range(4),
      'finite',
      FP.f32.multiplicationMatrixVectorInterval
    );
  },
  mat4x4_vec4_non_const: () => {
    return FP.f32.generateMatrixVectorToVectorCases(
      sparseMatrixF32Range(4, 4),
      sparseVectorF32Range(4),
      'unfiltered',
      FP.f32.multiplicationMatrixVectorInterval
    );
  },
  vec2_mat2x2_const: () => {
    return FP.f32.generateVectorMatrixToVectorCases(
      sparseVectorF32Range(2),
      sparseMatrixF32Range(2, 2),
      'finite',
      FP.f32.multiplicationVectorMatrixInterval
    );
  },
  vec2_mat2x2_non_const: () => {
    return FP.f32.generateVectorMatrixToVectorCases(
      sparseVectorF32Range(2),
      sparseMatrixF32Range(2, 2),
      'unfiltered',
      FP.f32.multiplicationVectorMatrixInterval
    );
  },
  vec2_mat3x2_const: () => {
    return FP.f32.generateVectorMatrixToVectorCases(
      sparseVectorF32Range(2),
      sparseMatrixF32Range(3, 2),
      'finite',
      FP.f32.multiplicationVectorMatrixInterval
    );
  },
  vec2_mat3x2_non_const: () => {
    return FP.f32.generateVectorMatrixToVectorCases(
      sparseVectorF32Range(2),
      sparseMatrixF32Range(3, 2),
      'unfiltered',
      FP.f32.multiplicationVectorMatrixInterval
    );
  },
  vec2_mat4x2_const: () => {
    return FP.f32.generateVectorMatrixToVectorCases(
      sparseVectorF32Range(2),
      sparseMatrixF32Range(4, 2),
      'finite',
      FP.f32.multiplicationVectorMatrixInterval
    );
  },
  vec2_mat4x2_non_const: () => {
    return FP.f32.generateVectorMatrixToVectorCases(
      sparseVectorF32Range(2),
      sparseMatrixF32Range(4, 2),
      'unfiltered',
      FP.f32.multiplicationVectorMatrixInterval
    );
  },
  vec3_mat2x3_const: () => {
    return FP.f32.generateVectorMatrixToVectorCases(
      sparseVectorF32Range(3),
      sparseMatrixF32Range(2, 3),
      'finite',
      FP.f32.multiplicationVectorMatrixInterval
    );
  },
  vec3_mat2x3_non_const: () => {
    return FP.f32.generateVectorMatrixToVectorCases(
      sparseVectorF32Range(3),
      sparseMatrixF32Range(2, 3),
      'unfiltered',
      FP.f32.multiplicationVectorMatrixInterval
    );
  },
  vec3_mat3x3_const: () => {
    return FP.f32.generateVectorMatrixToVectorCases(
      sparseVectorF32Range(3),
      sparseMatrixF32Range(3, 3),
      'finite',
      FP.f32.multiplicationVectorMatrixInterval
    );
  },
  vec3_mat3x3_non_const: () => {
    return FP.f32.generateVectorMatrixToVectorCases(
      sparseVectorF32Range(3),
      sparseMatrixF32Range(3, 3),
      'unfiltered',
      FP.f32.multiplicationVectorMatrixInterval
    );
  },
  vec3_mat4x3_const: () => {
    return FP.f32.generateVectorMatrixToVectorCases(
      sparseVectorF32Range(3),
      sparseMatrixF32Range(4, 3),
      'finite',
      FP.f32.multiplicationVectorMatrixInterval
    );
  },
  vec3_mat4x3_non_const: () => {
    return FP.f32.generateVectorMatrixToVectorCases(
      sparseVectorF32Range(3),
      sparseMatrixF32Range(4, 3),
      'unfiltered',
      FP.f32.multiplicationVectorMatrixInterval
    );
  },
  vec4_mat2x4_const: () => {
    return FP.f32.generateVectorMatrixToVectorCases(
      sparseVectorF32Range(4),
      sparseMatrixF32Range(2, 4),
      'finite',
      FP.f32.multiplicationVectorMatrixInterval
    );
  },
  vec4_mat2x4_non_const: () => {
    return FP.f32.generateVectorMatrixToVectorCases(
      sparseVectorF32Range(4),
      sparseMatrixF32Range(2, 4),
      'unfiltered',
      FP.f32.multiplicationVectorMatrixInterval
    );
  },
  vec4_mat3x4_const: () => {
    return FP.f32.generateVectorMatrixToVectorCases(
      sparseVectorF32Range(4),
      sparseMatrixF32Range(3, 4),
      'finite',
      FP.f32.multiplicationVectorMatrixInterval
    );
  },
  vec4_mat3x4_non_const: () => {
    return FP.f32.generateVectorMatrixToVectorCases(
      sparseVectorF32Range(4),
      sparseMatrixF32Range(3, 4),
      'unfiltered',
      FP.f32.multiplicationVectorMatrixInterval
    );
  },
  vec4_mat4x4_const: () => {
    return FP.f32.generateVectorMatrixToVectorCases(
      sparseVectorF32Range(4),
      sparseMatrixF32Range(4, 4),
      'finite',
      FP.f32.multiplicationVectorMatrixInterval
    );
  },
  vec4_mat4x4_non_const: () => {
    return FP.f32.generateVectorMatrixToVectorCases(
      sparseVectorF32Range(4),
      sparseMatrixF32Range(4, 4),
      'unfiltered',
      FP.f32.multiplicationVectorMatrixInterval
    );
  },
  subtraction_mat2x2_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(2, 2),
      sparseMatrixF32Range(2, 2),
      'finite',
      FP.f32.subtractionMatrixMatrixInterval
    );
  },
  subtraction_mat2x2_non_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(2, 2),
      sparseMatrixF32Range(2, 2),
      'unfiltered',
      FP.f32.subtractionMatrixMatrixInterval
    );
  },
});

g.test('matrix_vector')
  .specURL('https://www.w3.org/TR/WGSL/#floating-point-evaluation')
  .desc(
    `
Expression: x * y, where x is a matrix and y is a vector
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
        ? `mat${cols}x${rows}_vec${cols}_const`
        : `mat${cols}x${rows}_vec${cols}_non_const`
    );

    await run(
      t,
      binary('*'),
      [TypeMat(cols, rows, TypeF32), TypeVec(cols, TypeF32)],
      TypeVec(rows, TypeF32),
      t.params,
      cases
    );
  });

g.test('vector_matrix')
  .specURL('https://www.w3.org/TR/WGSL/#floating-point-evaluation')
  .desc(
    `
Expression: x * y, where x is a vector and y is is a matrix
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
        ? `vec${rows}_mat${cols}x${rows}_const`
        : `vec${rows}_mat${cols}x${rows}_non_const`
    );

    await run(
      t,
      binary('*'),
      [TypeVec(rows, TypeF32), TypeMat(cols, rows, TypeF32)],
      TypeVec(cols, TypeF32),
      t.params,
      cases
    );
  });

g.test('vector_matrix_compound')
  .specURL('https://www.w3.org/TR/WGSL/#floating-point-evaluation')
  .desc(
    `
Expression: x *= y, where x is a vector and y is is a matrix
Accuracy: Correctly rounded
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('dim', [2, 3, 4]))
  .fn(async t => {
    const cols = t.params.dim;
    const rows = t.params.dim;
    const cases = await d.get(
      t.params.inputSource === 'const'
        ? `vec${rows}_mat${cols}x${rows}_const`
        : `vec${rows}_mat${cols}x${rows}_non_const`
    );

    await run(
      t,
      compoundBinary('*='),
      [TypeVec(rows, TypeF32), TypeMat(cols, rows, TypeF32)],
      TypeVec(cols, TypeF32),
      t.params,
      cases
    );
  });
