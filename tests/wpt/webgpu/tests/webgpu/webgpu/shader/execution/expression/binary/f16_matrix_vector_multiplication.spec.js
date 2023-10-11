/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Execution Tests for matrix-vector and vector-matrix f16 multiplication expression
`;
import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../gpu_test.js';
import { TypeF16, TypeMat, TypeVec } from '../../../../util/conversion.js';
import { FP } from '../../../../util/floating_point.js';
import { sparseMatrixF16Range, sparseVectorF16Range } from '../../../../util/math.js';
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
          return FP.f16.generateMatrixVectorToVectorCases(
            sparseMatrixF16Range(cols, rows),
            sparseVectorF16Range(cols),
            nonConst ? 'unfiltered' : 'finite',
            FP.f16.multiplicationMatrixVectorInterval
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
          return FP.f16.generateVectorMatrixToVectorCases(
            sparseVectorF16Range(rows),
            sparseMatrixF16Range(cols, rows),
            nonConst ? 'unfiltered' : 'finite',
            FP.f16.multiplicationVectorMatrixInterval
          );
        },
      }))
    )
  )
  .reduce((a, b) => ({ ...a, ...b }), {});

export const d = makeCaseCache('binary/f16_matrix_vector_multiplication', {
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
  .beforeAllSubcases(t => {
    t.selectDeviceOrSkipTestCase({ requiredFeatures: ['shader-f16'] });
  })
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
      [TypeMat(cols, rows, TypeF16), TypeVec(cols, TypeF16)],
      TypeVec(rows, TypeF16),
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
  .beforeAllSubcases(t => {
    t.selectDeviceOrSkipTestCase({ requiredFeatures: ['shader-f16'] });
  })
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
      [TypeVec(rows, TypeF16), TypeMat(cols, rows, TypeF16)],
      TypeVec(cols, TypeF16),
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
  .beforeAllSubcases(t => {
    t.selectDeviceOrSkipTestCase({ requiredFeatures: ['shader-f16'] });
  })
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
      [TypeVec(rows, TypeF16), TypeMat(cols, rows, TypeF16)],
      TypeVec(cols, TypeF16),
      t.params,
      cases
    );
  });
