/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution Tests for the abstract int arithmetic binary expression operations
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../gpu_test.js';
import { Type } from '../../../../util/conversion.js';
import { onlyConstInputSource, run } from '../expression.js';

import { d } from './ai_arithmetic.cache.js';
import { abstractIntBinary } from './binary.js';

export const g = makeTestGroup(GPUTest);

g.test('addition').
specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr').
desc(
  `
Expression: x + y
`
).
params((u) =>
u.
combine('inputSource', onlyConstInputSource).
combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const cases = await d.get('addition');
  await run(
    t,
    abstractIntBinary('+'),
    [Type.abstractInt, Type.abstractInt],
    Type.abstractInt,
    t.params,
    cases
  );
});

g.test('addition_scalar_vector').
specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr').
desc(
  `
Expression: x + y
`
).
params((u) =>
u.combine('inputSource', onlyConstInputSource).combine('vectorize_rhs', [2, 3, 4])
).
fn(async (t) => {
  const vec_size = t.params.vectorize_rhs;
  const vec_type = Type.vec(vec_size, Type.abstractInt);
  const cases = await d.get(`addition_scalar_vector${vec_size}`);
  await run(t, abstractIntBinary('+'), [Type.abstractInt, vec_type], vec_type, t.params, cases);
});

g.test('addition_vector_scalar').
specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr').
desc(
  `
Expression: x + y
`
).
params((u) =>
u.combine('inputSource', onlyConstInputSource).combine('vectorize_lhs', [2, 3, 4])
).
fn(async (t) => {
  const vec_size = t.params.vectorize_lhs;
  const vec_type = Type.vec(vec_size, Type.abstractInt);
  const cases = await d.get(`addition_vector${vec_size}_scalar`);
  await run(t, abstractIntBinary('+'), [vec_type, Type.abstractInt], vec_type, t.params, cases);
});

g.test('division').
specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr').
desc(
  `
Expression: x / y
`
).
params((u) =>
u.
combine('inputSource', onlyConstInputSource).
combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const cases = await d.get('division');
  await run(
    t,
    abstractIntBinary('/'),
    [Type.abstractInt, Type.abstractInt],
    Type.abstractInt,
    t.params,
    cases
  );
});

g.test('division_scalar_vector').
specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr').
desc(
  `
Expression: x / y
`
).
params((u) =>
u.combine('inputSource', onlyConstInputSource).combine('vectorize_rhs', [2, 3, 4])
).
fn(async (t) => {
  const vec_size = t.params.vectorize_rhs;
  const vec_type = Type.vec(vec_size, Type.abstractInt);
  const cases = await d.get(`division_scalar_vector${vec_size}`);
  await run(t, abstractIntBinary('/'), [Type.abstractInt, vec_type], vec_type, t.params, cases);
});

g.test('division_vector_scalar').
specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr').
desc(
  `
Expression: x / y
`
).
params((u) =>
u.combine('inputSource', onlyConstInputSource).combine('vectorize_lhs', [2, 3, 4])
).
fn(async (t) => {
  const vec_size = t.params.vectorize_lhs;
  const vec_type = Type.vec(vec_size, Type.abstractInt);
  const cases = await d.get(`division_vector${vec_size}_scalar`);
  await run(t, abstractIntBinary('/'), [vec_type, Type.abstractInt], vec_type, t.params, cases);
});

g.test('multiplication').
specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr').
desc(
  `
Expression: x * y
`
).
params((u) =>
u.
combine('inputSource', onlyConstInputSource).
combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const cases = await d.get('multiplication');
  await run(
    t,
    abstractIntBinary('*'),
    [Type.abstractInt, Type.abstractInt],
    Type.abstractInt,
    t.params,
    cases
  );
});

g.test('multiplication_scalar_vector').
specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr').
desc(
  `
Expression: x * y
`
).
params((u) =>
u.combine('inputSource', onlyConstInputSource).combine('vectorize_rhs', [2, 3, 4])
).
fn(async (t) => {
  const vec_size = t.params.vectorize_rhs;
  const vec_type = Type.vec(vec_size, Type.abstractInt);
  const cases = await d.get(`multiplication_scalar_vector${vec_size}`);
  await run(t, abstractIntBinary('*'), [Type.abstractInt, vec_type], vec_type, t.params, cases);
});

g.test('multiplication_vector_scalar').
specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr').
desc(
  `
Expression: x * y
`
).
params((u) =>
u.combine('inputSource', onlyConstInputSource).combine('vectorize_lhs', [2, 3, 4])
).
fn(async (t) => {
  const vec_size = t.params.vectorize_lhs;
  const vec_type = Type.vec(vec_size, Type.abstractInt);
  const cases = await d.get(`multiplication_vector${vec_size}_scalar`);
  await run(t, abstractIntBinary('*'), [vec_type, Type.abstractInt], vec_type, t.params, cases);
});

g.test('remainder').
specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr').
desc(
  `
Expression: x % y
`
).
params((u) =>
u.
combine('inputSource', onlyConstInputSource).
combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const cases = await d.get('remainder');
  await run(
    t,
    abstractIntBinary('%'),
    [Type.abstractInt, Type.abstractInt],
    Type.abstractInt,
    t.params,
    cases
  );
});

g.test('remainder_scalar_vector').
specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr').
desc(
  `
Expression: x % y
`
).
params((u) =>
u.combine('inputSource', onlyConstInputSource).combine('vectorize_rhs', [2, 3, 4])
).
fn(async (t) => {
  const vec_size = t.params.vectorize_rhs;
  const vec_type = Type.vec(vec_size, Type.abstractInt);
  const cases = await d.get(`remainder_scalar_vector${vec_size}`);
  await run(t, abstractIntBinary('%'), [Type.abstractInt, vec_type], vec_type, t.params, cases);
});

g.test('remainder_vector_scalar').
specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr').
desc(
  `
Expression: x % y
`
).
params((u) =>
u.combine('inputSource', onlyConstInputSource).combine('vectorize_lhs', [2, 3, 4])
).
fn(async (t) => {
  const vec_size = t.params.vectorize_lhs;
  const vec_type = Type.vec(vec_size, Type.abstractInt);
  const cases = await d.get(`remainder_vector${vec_size}_scalar`);
  await run(t, abstractIntBinary('%'), [vec_type, Type.abstractInt], vec_type, t.params, cases);
});

g.test('subtraction').
specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr').
desc(
  `
Expression: x - y
`
).
params((u) =>
u.
combine('inputSource', onlyConstInputSource).
combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const cases = await d.get('subtraction');
  await run(
    t,
    abstractIntBinary('-'),
    [Type.abstractInt, Type.abstractInt],
    Type.abstractInt,
    t.params,
    cases
  );
});

g.test('subtraction_scalar_vector').
specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr').
desc(
  `
Expression: x - y
`
).
params((u) =>
u.combine('inputSource', onlyConstInputSource).combine('vectorize_rhs', [2, 3, 4])
).
fn(async (t) => {
  const vec_size = t.params.vectorize_rhs;
  const vec_type = Type.vec(vec_size, Type.abstractInt);
  const cases = await d.get(`subtraction_scalar_vector${vec_size}`);
  await run(t, abstractIntBinary('-'), [Type.abstractInt, vec_type], vec_type, t.params, cases);
});

g.test('subtraction_vector_scalar').
specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr').
desc(
  `
Expression: x - y
`
).
params((u) =>
u.combine('inputSource', onlyConstInputSource).combine('vectorize_lhs', [2, 3, 4])
).
fn(async (t) => {
  const vec_size = t.params.vectorize_lhs;
  const vec_type = Type.vec(vec_size, Type.abstractInt);
  const cases = await d.get(`subtraction_vector${vec_size}_scalar`);
  await run(t, abstractIntBinary('-'), [vec_type, Type.abstractInt], vec_type, t.params, cases);
});