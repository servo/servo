/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution Tests for matrix-matrix f16 multiplication expression
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../gpu_test.js';
import { TypeF16, TypeMat } from '../../../../util/conversion.js';
import { allInputSources, run } from '../expression.js';

import { binary, compoundBinary } from './binary.js';
import { d } from './f16_matrix_matrix_multiplication.cache.js';

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
combine('inputSource', allInputSources).
combine('common_dim', [2, 3, 4]).
combine('x_rows', [2, 3, 4]).
combine('y_cols', [2, 3, 4])
).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase({ requiredFeatures: ['shader-f16'] });
}).
fn(async (t) => {
  const x_cols = t.params.common_dim;
  const x_rows = t.params.x_rows;
  const y_cols = t.params.y_cols;
  const y_rows = t.params.common_dim;

  const cases = await d.get(
    t.params.inputSource === 'const' ?
    `mat${x_cols}x${x_rows}_mat${y_cols}x${y_rows}_const` :
    `mat${x_cols}x${x_rows}_mat${y_cols}x${y_rows}_non_const`
  );
  await run(
    t,
    binary('*'),
    [TypeMat(x_cols, x_rows, TypeF16), TypeMat(y_cols, y_rows, TypeF16)],
    TypeMat(y_cols, x_rows, TypeF16),
    t.params,
    cases
  );
});

g.test('matrix_matrix_compound').
specURL('https://www.w3.org/TR/WGSL/#floating-point-evaluation').
desc(
  `
Expression: x *= y, where x is a matrix and y is a matrix
Accuracy: Correctly rounded
`
).
params((u) =>
u.
combine('inputSource', allInputSources).
combine('common_dim', [2, 3, 4]).
combine('x_rows', [2, 3, 4])
).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase({ requiredFeatures: ['shader-f16'] });
}).
fn(async (t) => {
  const x_cols = t.params.common_dim;
  const x_rows = t.params.x_rows;
  const y_cols = x_cols;
  const y_rows = t.params.common_dim;

  const cases = await d.get(
    t.params.inputSource === 'const' ?
    `mat${x_cols}x${x_rows}_mat${y_cols}x${y_rows}_const` :
    `mat${x_cols}x${x_rows}_mat${y_cols}x${y_rows}_non_const`
  );
  await run(
    t,
    compoundBinary('*='),
    [TypeMat(x_cols, x_rows, TypeF16), TypeMat(y_cols, y_rows, TypeF16)],
    TypeMat(y_cols, x_rows, TypeF16),
    t.params,
    cases
  );
});