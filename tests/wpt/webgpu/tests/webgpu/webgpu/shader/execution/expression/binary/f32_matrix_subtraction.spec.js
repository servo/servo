/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution Tests for matrix f32 subtraction expression
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../gpu_test.js';
import { Type } from '../../../../util/conversion.js';
import { allInputSources, run } from '../expression.js';

import { binary, compoundBinary } from './binary.js';
import { d } from './f32_matrix_subtraction.cache.js';

export const g = makeTestGroup(GPUTest);

g.test('matrix').
specURL('https://www.w3.org/TR/WGSL/#floating-point-evaluation').
desc(
  `
Expression: x - y, where x and y are matrices
Accuracy: Correctly rounded
`
).
params((u) =>
u.
combine('inputSource', allInputSources).
combine('cols', [2, 3, 4]).
combine('rows', [2, 3, 4])
).
fn(async (t) => {
  const cols = t.params.cols;
  const rows = t.params.rows;
  const cases = await d.get(
    t.params.inputSource === 'const' ? `mat${cols}x${rows}_const` : `mat${cols}x${rows}_non_const`
  );
  await run(
    t,
    binary('-'),
    [Type.mat(cols, rows, Type.f32), Type.mat(cols, rows, Type.f32)],
    Type.mat(cols, rows, Type.f32),
    t.params,
    cases
  );
});

g.test('matrix_compound').
specURL('https://www.w3.org/TR/WGSL/#floating-point-evaluation').
desc(
  `
Expression: x -= y, where x and y are matrices
Accuracy: Correctly rounded
`
).
params((u) =>
u.
combine('inputSource', allInputSources).
combine('cols', [2, 3, 4]).
combine('rows', [2, 3, 4])
).
fn(async (t) => {
  const cols = t.params.cols;
  const rows = t.params.rows;
  const cases = await d.get(
    t.params.inputSource === 'const' ? `mat${cols}x${rows}_const` : `mat${cols}x${rows}_non_const`
  );
  await run(
    t,
    compoundBinary('-='),
    [Type.mat(cols, rows, Type.f32), Type.mat(cols, rows, Type.f32)],
    Type.mat(cols, rows, Type.f32),
    t.params,
    cases
  );
});