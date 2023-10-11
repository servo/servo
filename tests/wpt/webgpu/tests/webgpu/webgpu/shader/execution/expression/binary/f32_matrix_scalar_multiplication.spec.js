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

// Cases: matCxR_scalar_[non_]const
const mat_scalar_cases = [2, 3, 4]
  .flatMap(cols =>
    [2, 3, 4].flatMap(rows =>
      [true, false].map(nonConst => ({
        [`mat${cols}x${rows}_scalar_${nonConst ? 'non_const' : 'const'}`]: () => {
          return FP.f32.generateMatrixScalarToMatrixCases(
            sparseMatrixF32Range(cols, rows),
            sparseF32Range(),
            nonConst ? 'unfiltered' : 'finite',
            FP.f32.multiplicationMatrixScalarInterval
          );
        },
      }))
    )
  )
  .reduce((a, b) => ({ ...a, ...b }), {});

// Cases: scalar_matCxR_[non_]const
const scalar_mat_cases = [2, 3, 4]
  .flatMap(cols =>
    [2, 3, 4].flatMap(rows =>
      [true, false].map(nonConst => ({
        [`scalar_mat${cols}x${rows}_${nonConst ? 'non_const' : 'const'}`]: () => {
          return FP.f32.generateScalarMatrixToMatrixCases(
            sparseF32Range(),
            sparseMatrixF32Range(cols, rows),
            nonConst ? 'unfiltered' : 'finite',
            FP.f32.multiplicationScalarMatrixInterval
          );
        },
      }))
    )
  )
  .reduce((a, b) => ({ ...a, ...b }), {});

export const d = makeCaseCache('binary/f32_matrix_scalar_multiplication', {
  ...mat_scalar_cases,
  ...scalar_mat_cases,
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
