/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { assert } from '../../../../../common/util/util.js';import { FP } from '../../../../util/floating_point.js';
import { selectNCases } from '../case.js';
import { makeCaseCache } from '../case_cache.js';

import { getAdditionAFInterval, kSparseMatrixAFValues } from './af_data.js';

const additionMatrixMatrixInterval = (
lhs,
rhs) =>
{
  assert(lhs.length === rhs.length, 'lhs and rhs have different number of columns');
  assert(rhs[0].length === rhs[0].length, 'lhs and rhs have different number of rows');
  const cols = lhs.length;
  const rows = rhs[0].length;

  const result = [...Array(cols)].map((_) => [...Array(rows)]);
  for (let i = 0; i < cols; i++) {
    for (let j = 0; j < rows; j++) {
      result[i][j] = getAdditionAFInterval(lhs[i][j], rhs[i][j]);
    }
  }
  return FP.abstract.toMatrix(result);
};

// Cases: matCxR
const mat_cases = [2, 3, 4].
flatMap((cols) =>
[2, 3, 4].map((rows) => ({
  [`mat${cols}x${rows}`]: () => {
    return selectNCases(
      'binary/af_matrix_addition',
      50,
      FP.abstract.generateMatrixPairToMatrixCases(
        kSparseMatrixAFValues[cols][rows],
        kSparseMatrixAFValues[cols][rows],
        'finite',
        additionMatrixMatrixInterval
      )
    );
  }
}))
).
reduce((a, b) => ({ ...a, ...b }), {});

export const d = makeCaseCache('binary/af_matrix_addition', mat_cases);