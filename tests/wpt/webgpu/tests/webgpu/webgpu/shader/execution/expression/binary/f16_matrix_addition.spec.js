/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Execution Tests for matrix f16 addition expression
`;
import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../gpu_test.js';
import { TypeF16, TypeMat } from '../../../../util/conversion.js';
import { FP } from '../../../../util/floating_point.js';
import { sparseMatrixF16Range } from '../../../../util/math.js';
import { makeCaseCache } from '../case_cache.js';
import { allInputSources, run } from '../expression.js';

import { binary, compoundBinary } from './binary.js';

export const g = makeTestGroup(GPUTest);

// Cases: matCxR_[non_]const
const mat_cases = [2, 3, 4]
  .flatMap(cols =>
    [2, 3, 4].flatMap(rows =>
      [true, false].map(nonConst => ({
        [`mat${cols}x${rows}_${nonConst ? 'non_const' : 'const'}`]: () => {
          return FP.f16.generateMatrixPairToMatrixCases(
            sparseMatrixF16Range(cols, rows),
            sparseMatrixF16Range(cols, rows),
            nonConst ? 'unfiltered' : 'finite',
            FP.f16.additionMatrixMatrixInterval
          );
        },
      }))
    )
  )
  .reduce((a, b) => ({ ...a, ...b }), {});

export const d = makeCaseCache('binary/f16_matrix_addition', mat_cases);

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
  .beforeAllSubcases(t => {
    t.selectDeviceOrSkipTestCase({ requiredFeatures: ['shader-f16'] });
  })
  .fn(async t => {
    const cols = t.params.cols;
    const rows = t.params.rows;
    const cases = await d.get(
      t.params.inputSource === 'const' ? `mat${cols}x${rows}_const` : `mat${cols}x${rows}_non_const`
    );

    await run(
      t,
      binary('+'),
      [TypeMat(cols, rows, TypeF16), TypeMat(cols, rows, TypeF16)],
      TypeMat(cols, rows, TypeF16),
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
  .beforeAllSubcases(t => {
    t.selectDeviceOrSkipTestCase({ requiredFeatures: ['shader-f16'] });
  })
  .fn(async t => {
    const cols = t.params.cols;
    const rows = t.params.rows;
    const cases = await d.get(
      t.params.inputSource === 'const' ? `mat${cols}x${rows}_const` : `mat${cols}x${rows}_non_const`
    );

    await run(
      t,
      compoundBinary('+='),
      [TypeMat(cols, rows, TypeF16), TypeMat(cols, rows, TypeF16)],
      TypeMat(cols, rows, TypeF16),
      t.params,
      cases
    );
  });
