/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution Tests for matrix-vector and vector-matrix AbstractFloat multiplication expression
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../gpu_test.js';
import { Type } from '../../../../util/conversion.js';
import { onlyConstInputSource, run } from '../expression.js';

import { d } from './af_matrix_vector_multiplication.cache.js';
import { abstractFloatBinary } from './binary.js';

export const g = makeTestGroup(GPUTest);

g.test('matrix_vector').
specURL('https://www.w3.org/TR/WGSL/#floating-point-evaluation').
desc(
  `
Expression: x * y, where x is a matrix and y is a vector
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
  const cases = await d.get(`mat${cols}x${rows}_vec${cols}`);
  await run(
    t,
    abstractFloatBinary('*'),
    [Type.mat(cols, rows, Type.abstractFloat), Type.vec(cols, Type.abstractFloat)],
    Type.vec(rows, Type.abstractFloat),
    t.params,
    cases
  );
});

g.test('vector_matrix').
specURL('https://www.w3.org/TR/WGSL/#floating-point-evaluation').
desc(
  `
Expression: x * y, where x is a vector and y is is a matrix
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
  const cases = await d.get(`vec${rows}_mat${cols}x${rows}`);
  await run(
    t,
    abstractFloatBinary('*'),
    [Type.vec(rows, Type.abstractFloat), Type.mat(cols, rows, Type.abstractFloat)],
    Type.vec(cols, Type.abstractFloat),
    t.params,
    cases
  );
});