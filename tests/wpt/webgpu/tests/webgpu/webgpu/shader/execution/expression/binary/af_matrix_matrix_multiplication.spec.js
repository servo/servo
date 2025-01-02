/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution Tests for matrix-matrix AbstractFloat multiplication expression
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../gpu_test.js';
import { Type } from '../../../../util/conversion.js';
import { onlyConstInputSource, run } from '../expression.js';

import { d } from './af_matrix_matrix_multiplication.cache.js';
import { abstractFloatBinary } from './binary.js';

export const g = makeTestGroup(GPUTest);

g.test('matrix_matrix').
specURL('https://www.w3.org/TR/WGSL/#floating-point-evaluation').
desc(
  `
Expression: x * y, where x is a matrix and y is a matrix
Accuracy: Correctly rounded
`
).
params((u) =>
u.
combine('inputSource', onlyConstInputSource).
combine('common_dim', [2, 3, 4]).
combine('x_rows', [2, 3, 4]).
combine('y_cols', [2, 3, 4])
).
fn(async (t) => {
  const x_cols = t.params.common_dim;
  const x_rows = t.params.x_rows;
  const y_cols = t.params.y_cols;
  const y_rows = t.params.common_dim;

  const cases = await d.get(`mat${x_cols}x${x_rows}_mat${y_cols}x${y_rows}`);
  await run(
    t,
    abstractFloatBinary('*'),
    [Type.mat(x_cols, x_rows, Type.abstractFloat), Type.mat(y_cols, y_rows, Type.abstractFloat)],
    Type.mat(y_cols, x_rows, Type.abstractFloat),
    t.params,
    cases
  );
});