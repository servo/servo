/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Execution Tests for matrix AbstractFloat subtraction expression
`;
import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../gpu_test.js';
import { TypeAbstractFloat, TypeMat } from '../../../../util/conversion.js';
import { FP } from '../../../../util/floating_point.js';
import { sparseMatrixF64Range } from '../../../../util/math.js';
import { makeCaseCache } from '../case_cache.js';
import { onlyConstInputSource, run } from '../expression.js';

import { abstractBinary } from './binary.js';

export const g = makeTestGroup(GPUTest);

// Cases: matCxR
const mat_cases = [2, 3, 4]
  .flatMap(cols =>
    [2, 3, 4].map(rows => ({
      [`mat${cols}x${rows}`]: () => {
        return FP.abstract.generateMatrixPairToMatrixCases(
          sparseMatrixF64Range(cols, rows),
          sparseMatrixF64Range(cols, rows),
          'finite',
          FP.abstract.subtractionMatrixMatrixInterval
        );
      },
    }))
  )
  .reduce((a, b) => ({ ...a, ...b }), {});

export const d = makeCaseCache('binary/af_matrix_subtraction', mat_cases);

g.test('matrix')
  .specURL('https://www.w3.org/TR/WGSL/#floating-point-evaluation')
  .desc(
    `
Expression: x - y, where x and y are matrices
Accuracy: Correctly rounded
`
  )
  .params(u =>
    u
      .combine('inputSource', onlyConstInputSource)
      .combine('cols', [2, 3, 4])
      .combine('rows', [2, 3, 4])
  )
  .fn(async t => {
    const cols = t.params.cols;
    const rows = t.params.rows;
    const cases = await d.get(`mat${cols}x${rows}`);
    await run(
      t,
      abstractBinary('-'),
      [TypeMat(cols, rows, TypeAbstractFloat), TypeMat(cols, rows, TypeAbstractFloat)],
      TypeMat(cols, rows, TypeAbstractFloat),
      t.params,
      cases
    );
  });
