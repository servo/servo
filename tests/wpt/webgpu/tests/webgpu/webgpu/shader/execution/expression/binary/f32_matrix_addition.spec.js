/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Execution Tests for matrix f32 addition expression
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

export const d = makeCaseCache('binary/f32_matrix_addition', {
  mat2x2_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(2, 2),
      sparseMatrixF32Range(2, 2),
      'finite',
      FP.f32.additionMatrixMatrixInterval
    );
  },
  mat2x2_non_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(2, 2),
      sparseMatrixF32Range(2, 2),
      'unfiltered',
      FP.f32.additionMatrixMatrixInterval
    );
  },
  mat2x3_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(2, 3),
      sparseMatrixF32Range(2, 3),
      'finite',
      FP.f32.additionMatrixMatrixInterval
    );
  },
  mat2x3_non_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(2, 3),
      sparseMatrixF32Range(2, 3),
      'unfiltered',
      FP.f32.additionMatrixMatrixInterval
    );
  },
  mat2x4_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(2, 4),
      sparseMatrixF32Range(2, 4),
      'finite',
      FP.f32.additionMatrixMatrixInterval
    );
  },
  mat2x4_non_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(2, 4),
      sparseMatrixF32Range(2, 4),
      'unfiltered',
      FP.f32.additionMatrixMatrixInterval
    );
  },
  mat3x2_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(3, 2),
      sparseMatrixF32Range(3, 2),
      'finite',
      FP.f32.additionMatrixMatrixInterval
    );
  },
  mat3x2_non_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(3, 2),
      sparseMatrixF32Range(3, 2),
      'unfiltered',
      FP.f32.additionMatrixMatrixInterval
    );
  },
  mat3x3_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(3, 3),
      sparseMatrixF32Range(3, 3),
      'finite',
      FP.f32.additionMatrixMatrixInterval
    );
  },
  mat3x3_non_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(3, 3),
      sparseMatrixF32Range(3, 3),
      'unfiltered',
      FP.f32.additionMatrixMatrixInterval
    );
  },
  mat3x4_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(3, 4),
      sparseMatrixF32Range(3, 4),
      'finite',
      FP.f32.additionMatrixMatrixInterval
    );
  },
  mat3x4_non_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(3, 4),
      sparseMatrixF32Range(3, 4),
      'unfiltered',
      FP.f32.additionMatrixMatrixInterval
    );
  },
  mat4x2_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(4, 2),
      sparseMatrixF32Range(4, 2),
      'finite',
      FP.f32.additionMatrixMatrixInterval
    );
  },
  mat4x2_non_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(4, 2),
      sparseMatrixF32Range(4, 2),
      'unfiltered',
      FP.f32.additionMatrixMatrixInterval
    );
  },
  mat4x3_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(4, 3),
      sparseMatrixF32Range(4, 3),
      'finite',
      FP.f32.additionMatrixMatrixInterval
    );
  },
  mat4x3_non_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(4, 3),
      sparseMatrixF32Range(4, 3),
      'unfiltered',
      FP.f32.additionMatrixMatrixInterval
    );
  },
  mat4x4_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(4, 4),
      sparseMatrixF32Range(4, 4),
      'finite',
      FP.f32.additionMatrixMatrixInterval
    );
  },
  mat4x4_non_const: () => {
    return FP.f32.generateMatrixPairToMatrixCases(
      sparseMatrixF32Range(4, 4),
      sparseMatrixF32Range(4, 4),
      'unfiltered',
      FP.f32.additionMatrixMatrixInterval
    );
  },
});

g.test('matrix')
  .specURL('https://www.w3.org/TR/WGSL/#floating-point-evaluation')
  .desc(
    `
Expression: x + y, where x and y are matrices
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
      t.params.inputSource === 'const' ? `mat${cols}x${rows}_const` : `mat${cols}x${rows}_non_const`
    );

    await run(
      t,
      binary('+'),
      [TypeMat(cols, rows, TypeF32), TypeMat(cols, rows, TypeF32)],
      TypeMat(cols, rows, TypeF32),
      t.params,
      cases
    );
  });

g.test('matrix_compound')
  .specURL('https://www.w3.org/TR/WGSL/#floating-point-evaluation')
  .desc(
    `
Expression: x =+ y, where x and y are matrices
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
      t.params.inputSource === 'const' ? `mat${cols}x${rows}_const` : `mat${cols}x${rows}_non_const`
    );

    await run(
      t,
      compoundBinary('+='),
      [TypeMat(cols, rows, TypeF32), TypeMat(cols, rows, TypeF32)],
      TypeMat(cols, rows, TypeF32),
      t.params,
      cases
    );
  });
