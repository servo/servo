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

// Cases: matCxR_vecC_[non_]const
const mat_vec_cases = [2, 3, 4]
  .flatMap(cols =>
    [2, 3, 4].flatMap(rows =>
      [true, false].map(nonConst => ({
        [`mat${cols}x${rows}_vec${cols}_${nonConst ? 'non_const' : 'const'}`]: () => {
          return FP.f32.generateMatrixVectorToVectorCases(
            sparseMatrixF32Range(cols, rows),
            sparseVectorF32Range(cols),
            nonConst ? 'unfiltered' : 'finite',
            FP.f32.multiplicationMatrixVectorInterval
          );
        },
      }))
    )
  )
  .reduce((a, b) => ({ ...a, ...b }), {});

// Cases: vecR_matCxR_[non_]const
const vec_mat_cases = [2, 3, 4]
  .flatMap(rows =>
    [2, 3, 4].flatMap(cols =>
      [true, false].map(nonConst => ({
        [`vec${rows}_mat${cols}x${rows}_${nonConst ? 'non_const' : 'const'}`]: () => {
          return FP.f32.generateVectorMatrixToVectorCases(
            sparseVectorF32Range(rows),
            sparseMatrixF32Range(cols, rows),
            nonConst ? 'unfiltered' : 'finite',
            FP.f32.multiplicationVectorMatrixInterval
          );
        },
      }))
    )
  )
  .reduce((a, b) => ({ ...a, ...b }), {});

export const d = makeCaseCache('binary/f32_matrix_vector_multiplication', {
  ...mat_vec_cases,
  ...vec_mat_cases,
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
