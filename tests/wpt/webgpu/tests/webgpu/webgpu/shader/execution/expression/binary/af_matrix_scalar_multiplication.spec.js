/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution Tests for matrix-scalar and scalar-matrix AbstractFloat multiplication expression
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../gpu_test.js';
import { Type } from '../../../../util/conversion.js';
import { onlyConstInputSource, run } from '../expression.js';

import { d } from './af_matrix_scalar_multiplication.cache.js';
import { abstractFloatBinary } from './binary.js';

export const g = makeTestGroup(GPUTest);

g.test('matrix_scalar').
specURL('https://www.w3.org/TR/WGSL/#floating-point-evaluation').
desc(
  `
Expression: x * y, where x is a matrix and y is a scalar
Accuracy: Correctly rounded
`
).
params((u) =>
u.
combine('inputSource', onlyConstInputSource).
combine('cols', [2, 3, 4]).
combine('rows', [2, 3, 4])
).
fn(async (t) => {
  const cols = t.params.cols;
  const rows = t.params.rows;
  const cases = await d.get(`mat${cols}x${rows}_scalar`);
  await run(
    t,
    abstractFloatBinary('*'),
    [Type.mat(cols, rows, Type.abstractFloat), Type.abstractFloat],
    Type.mat(cols, rows, Type.abstractFloat),
    t.params,
    cases
  );
});

g.test('scalar_matrix').
specURL('https://www.w3.org/TR/WGSL/#floating-point-evaluation').
desc(
  `
Expression: x * y, where x is a scalar and y is a matrix
Accuracy: Correctly rounded
`
).
params((u) =>
u.
combine('inputSource', onlyConstInputSource).
combine('cols', [2, 3, 4]).
combine('rows', [2, 3, 4])
).
fn(async (t) => {
  const cols = t.params.cols;
  const rows = t.params.rows;
  const cases = await d.get(`scalar_mat${cols}x${rows}`);
  await run(
    t,
    abstractFloatBinary('*'),
    [Type.abstractFloat, Type.mat(cols, rows, Type.abstractFloat)],
    Type.mat(cols, rows, Type.abstractFloat),
    t.params,
    cases
  );
});