/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution Tests for matrix indexing expressions
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';
import {
  MatrixValue,

  Type,
  abstractFloat,
  f32,
  vec } from
'../../../../../util/conversion.js';

import { allInputSources, basicExpressionBuilder, run } from '../../expression.js';

export const g = makeTestGroup(GPUTest);

g.test('concrete_float_column').
specURL('https://www.w3.org/TR/WGSL/#matrix-access-expr').
desc(`Test indexing a column vector from a concrete matrix`).
params((u) =>
u.
combine('inputSource', allInputSources).
combine('elementType', ['f32', 'f16']).
combine('indexType', ['i32', 'u32']).
combine('columns', [2, 3, 4]).
combine('rows', [2, 3, 4])
).
beforeAllSubcases((t) => {
  if (t.params.elementType === 'f16') {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn(async (t) => {
  const elementType = Type[t.params.elementType];
  const indexType = Type[t.params.indexType];
  const matrixType = Type.mat(t.params.columns, t.params.rows, elementType);
  const columnType = Type.vec(t.params.rows, elementType);
  const elements = [];
  for (let c = 0; c < t.params.columns; c++) {
    const column = [];
    for (let r = 0; r < t.params.rows; r++) {
      column.push(elementType.create((c + 1) * 10 + (r + 1)));
    }
    elements.push(column);
  }
  const vector = new MatrixValue(elements);
  const cases = [];
  for (let c = 0; c < t.params.columns; c++) {
    cases.push({
      input: [vector, indexType.create(c)],
      expected: vec(...elements[c])
    });
  }

  await run(
    t,
    basicExpressionBuilder((ops) => `${ops[0]}[${ops[1]}]`),
    [matrixType, indexType],
    columnType,
    t.params,
    cases
  );
});

g.test('concrete_float_element').
specURL('https://www.w3.org/TR/WGSL/#matrix-access-expr').
desc(`Test indexing a single element from a concrete matrix`).
params((u) =>
u.
combine('inputSource', allInputSources).
combine('elementType', ['f32', 'f16']).
combine('indexType', ['i32', 'u32']).
combine('columns', [2, 3, 4]).
combine('rows', [2, 3, 4])
).
beforeAllSubcases((t) => {
  if (t.params.elementType === 'f16') {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn(async (t) => {
  const elementType = Type[t.params.elementType];
  const indexType = Type[t.params.indexType];
  const matrixType = Type.mat(t.params.columns, t.params.rows, elementType);
  const columnValues = [];
  for (let c = 0; c < t.params.columns; c++) {
    const column = [];
    for (let r = 0; r < t.params.rows; r++) {
      column.push(elementType.create((c + 1) * 10 + (r + 1)));
    }
    columnValues.push(column);
  }
  const matrix = new MatrixValue(columnValues);
  const cases = [];
  for (let c = 0; c < t.params.columns; c++) {
    for (let r = 0; r < t.params.rows; r++) {
      cases.push({
        input: [matrix, indexType.create(c), indexType.create(r)],
        expected: columnValues[c][r]
      });
    }
  }

  await run(
    t,
    basicExpressionBuilder((ops) => `${ops[0]}[${ops[1]}][${ops[2]}]`),
    [matrixType, indexType, indexType],
    elementType,
    t.params,
    cases
  );
});

g.test('abstract_float_column').
specURL('https://www.w3.org/TR/WGSL/#matrix-access-expr').
desc(`Test indexing a column vector from a abstract-float matrix`).
params((u) =>
u.
combine('indexType', ['i32', 'u32']).
combine('columns', [2, 3, 4]).
combine('rows', [2, 3, 4])
).
fn(async (t) => {
  const indexType = Type[t.params.indexType];
  const matrixType = Type.mat(t.params.columns, t.params.rows, Type.abstractFloat);
  const vecfColumnType = Type.vec(t.params.rows, Type.f32);
  const values = [];
  for (let c = 0; c < t.params.columns; c++) {
    const column = [];
    for (let r = 0; r < t.params.rows; r++) {
      column.push((c + 1) * 10 + (r + 1));
    }
    values.push(column);
  }
  const matrix = new MatrixValue(
    values.map((column) => column.map((v) => abstractFloat(v * 0x100000000)))
  );
  const cases = [];
  for (let c = 0; c < t.params.columns; c++) {
    cases.push({
      input: [matrix, indexType.create(c)],
      expected: vec(...values[c].map((v) => f32(v)))
    });
  }

  await run(
    t,
    basicExpressionBuilder((ops) => `${ops[0]}[${ops[1]}] / 0x100000000`),
    [matrixType, indexType],
    vecfColumnType,
    { inputSource: 'const' },
    cases
  );
});

g.test('abstract_float_element').
specURL('https://www.w3.org/TR/WGSL/#matrix-access-expr').
desc(`Test indexing a single element from a abstract-float matrix`).
params((u) =>
u.
combine('indexType', ['i32', 'u32']).
combine('columns', [2, 3, 4]).
combine('rows', [2, 3, 4])
).
fn(async (t) => {
  const indexType = Type[t.params.indexType];
  const matrixType = Type.mat(t.params.columns, t.params.rows, Type.abstractFloat);
  const values = [];
  for (let c = 0; c < t.params.columns; c++) {
    const column = [];
    for (let r = 0; r < t.params.rows; r++) {
      column.push((c + 1) * 10 + (r + 1));
    }
    values.push(column);
  }
  const matrix = new MatrixValue(
    values.map((column) => column.map((v) => abstractFloat(v * 0x100000000)))
  );
  const cases = [];
  for (let c = 0; c < t.params.columns; c++) {
    for (let r = 0; r < t.params.rows; r++) {
      cases.push({
        input: [matrix, indexType.create(c), indexType.create(r)],
        expected: f32(values[c][r])
      });
    }
  }

  await run(
    t,
    basicExpressionBuilder((ops) => `${ops[0]}[${ops[1]}][${ops[2]}] / 0x100000000`),
    [matrixType, indexType, indexType],
    Type.f32,
    { inputSource: 'const' },
    cases
  );
});