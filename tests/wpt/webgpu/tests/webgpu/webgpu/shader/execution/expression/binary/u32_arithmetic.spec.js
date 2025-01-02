/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution Tests for the u32 arithmetic binary expression operations
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../gpu_test.js';
import { Type } from '../../../../util/conversion.js';
import { allInputSources, run } from '../expression.js';

import { binary, compoundBinary } from './binary.js';
import { d } from './u32_arithmetic.cache.js';

export const g = makeTestGroup(GPUTest);

g.test('addition').
specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr').
desc(
  `
Expression: x + y
`
).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const cases = await d.get('addition');
  await run(t, binary('+'), [Type.u32, Type.u32], Type.u32, t.params, cases);
});

g.test('addition_compound').
specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr').
desc(
  `
Expression: x += y
`
).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const cases = await d.get('addition');
  await run(t, compoundBinary('+='), [Type.u32, Type.u32], Type.u32, t.params, cases);
});

g.test('subtraction').
specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr').
desc(
  `
Expression: x - y
`
).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const cases = await d.get('subtraction');
  await run(t, binary('-'), [Type.u32, Type.u32], Type.u32, t.params, cases);
});

g.test('subtraction_compound').
specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr').
desc(
  `
Expression: x -= y
`
).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const cases = await d.get('subtraction');
  await run(t, compoundBinary('-='), [Type.u32, Type.u32], Type.u32, t.params, cases);
});

g.test('multiplication').
specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr').
desc(
  `
Expression: x * y
`
).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const cases = await d.get('multiplication');
  await run(t, binary('*'), [Type.u32, Type.u32], Type.u32, t.params, cases);
});

g.test('multiplication_compound').
specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr').
desc(
  `
Expression: x *= y
`
).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const cases = await d.get('multiplication');
  await run(t, compoundBinary('*='), [Type.u32, Type.u32], Type.u32, t.params, cases);
});

g.test('division').
specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr').
desc(
  `
Expression: x / y
`
).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const cases = await d.get(
    t.params.inputSource === 'const' ? 'division_const' : 'division_non_const'
  );
  await run(t, binary('/'), [Type.u32, Type.u32], Type.u32, t.params, cases);
});

g.test('division_compound').
specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr').
desc(
  `
Expression: x /= y
`
).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const cases = await d.get(
    t.params.inputSource === 'const' ? 'division_const' : 'division_non_const'
  );
  await run(t, compoundBinary('/='), [Type.u32, Type.u32], Type.u32, t.params, cases);
});

g.test('remainder').
specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr').
desc(
  `
Expression: x % y
`
).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const cases = await d.get(
    t.params.inputSource === 'const' ? 'remainder_const' : 'remainder_non_const'
  );
  await run(t, binary('%'), [Type.u32, Type.u32], Type.u32, t.params, cases);
});

g.test('remainder_compound').
specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr').
desc(
  `
Expression: x %= y
`
).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const cases = await d.get(
    t.params.inputSource === 'const' ? 'remainder_const' : 'remainder_non_const'
  );
  await run(t, compoundBinary('%='), [Type.u32, Type.u32], Type.u32, t.params, cases);
});

g.test('addition_scalar_vector').
specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr').
desc(
  `
Expression: x + y
`
).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize_rhs', [2, 3, 4])
).
fn(async (t) => {
  const vec_size = t.params.vectorize_rhs;
  const vec_type = Type.vec(vec_size, Type.u32);
  const cases = await d.get(`addition_scalar_vector${vec_size}`);
  await run(t, binary('+'), [Type.u32, vec_type], vec_type, t.params, cases);
});

g.test('addition_vector_scalar').
specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr').
desc(
  `
Expression: x + y
`
).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize_lhs', [2, 3, 4])
).
fn(async (t) => {
  const vec_size = t.params.vectorize_lhs;
  const vec_type = Type.vec(vec_size, Type.u32);
  const cases = await d.get(`addition_vector${vec_size}_scalar`);
  await run(t, binary('+'), [vec_type, Type.u32], vec_type, t.params, cases);
});

g.test('addition_vector_scalar_compound').
specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr').
desc(
  `
Expression: x += y
`
).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize_lhs', [2, 3, 4])
).
fn(async (t) => {
  const vec_size = t.params.vectorize_lhs;
  const vec_type = Type.vec(vec_size, Type.u32);
  const cases = await d.get(`addition_vector${vec_size}_scalar`);
  await run(t, compoundBinary('+='), [vec_type, Type.u32], vec_type, t.params, cases);
});

g.test('subtraction_scalar_vector').
specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr').
desc(
  `
Expression: x - y
`
).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize_rhs', [2, 3, 4])
).
fn(async (t) => {
  const vec_size = t.params.vectorize_rhs;
  const vec_type = Type.vec(vec_size, Type.u32);
  const cases = await d.get(`subtraction_scalar_vector${vec_size}`);
  await run(t, binary('-'), [Type.u32, vec_type], vec_type, t.params, cases);
});

g.test('subtraction_vector_scalar').
specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr').
desc(
  `
Expression: x - y
`
).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize_lhs', [2, 3, 4])
).
fn(async (t) => {
  const vec_size = t.params.vectorize_lhs;
  const vec_type = Type.vec(vec_size, Type.u32);
  const cases = await d.get(`subtraction_vector${vec_size}_scalar`);
  await run(t, binary('-'), [vec_type, Type.u32], vec_type, t.params, cases);
});

g.test('subtraction_vector_scalar_compound').
specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr').
desc(
  `
Expression: x -= y
`
).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize_lhs', [2, 3, 4])
).
fn(async (t) => {
  const vec_size = t.params.vectorize_lhs;
  const vec_type = Type.vec(vec_size, Type.u32);
  const cases = await d.get(`subtraction_vector${vec_size}_scalar`);
  await run(t, compoundBinary('-='), [vec_type, Type.u32], vec_type, t.params, cases);
});

g.test('multiplication_scalar_vector').
specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr').
desc(
  `
Expression: x * y
`
).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize_rhs', [2, 3, 4])
).
fn(async (t) => {
  const vec_size = t.params.vectorize_rhs;
  const vec_type = Type.vec(vec_size, Type.u32);
  const cases = await d.get(`multiplication_scalar_vector${vec_size}`);
  await run(t, binary('*'), [Type.u32, vec_type], vec_type, t.params, cases);
});

g.test('multiplication_vector_scalar').
specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr').
desc(
  `
Expression: x * y
`
).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize_lhs', [2, 3, 4])
).
fn(async (t) => {
  const vec_size = t.params.vectorize_lhs;
  const vec_type = Type.vec(vec_size, Type.u32);
  const cases = await d.get(`multiplication_vector${vec_size}_scalar`);
  await run(t, binary('*'), [vec_type, Type.u32], vec_type, t.params, cases);
});

g.test('multiplication_vector_scalar_compound').
specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr').
desc(
  `
Expression: x *= y
`
).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize_lhs', [2, 3, 4])
).
fn(async (t) => {
  const vec_size = t.params.vectorize_lhs;
  const vec_type = Type.vec(vec_size, Type.u32);
  const cases = await d.get(`multiplication_vector${vec_size}_scalar`);
  await run(t, compoundBinary('*='), [vec_type, Type.u32], vec_type, t.params, cases);
});

g.test('division_scalar_vector').
specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr').
desc(
  `
Expression: x / y
`
).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize_rhs', [2, 3, 4])
).
fn(async (t) => {
  const vec_size = t.params.vectorize_rhs;
  const vec_type = Type.vec(vec_size, Type.u32);
  const source = t.params.inputSource === 'const' ? 'const' : 'non_const';
  const cases = await d.get(`division_scalar_vector${vec_size}_${source}`);
  await run(t, binary('/'), [Type.u32, vec_type], vec_type, t.params, cases);
});

g.test('division_vector_scalar').
specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr').
desc(
  `
Expression: x / y
`
).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize_lhs', [2, 3, 4])
).
fn(async (t) => {
  const vec_size = t.params.vectorize_lhs;
  const vec_type = Type.vec(vec_size, Type.u32);
  const source = t.params.inputSource === 'const' ? 'const' : 'non_const';
  const cases = await d.get(`division_vector${vec_size}_scalar_${source}`);
  await run(t, binary('/'), [vec_type, Type.u32], vec_type, t.params, cases);
});

g.test('division_vector_scalar_compound').
specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr').
desc(
  `
Expression: x /= y
`
).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize_lhs', [2, 3, 4])
).
fn(async (t) => {
  const vec_size = t.params.vectorize_lhs;
  const vec_type = Type.vec(vec_size, Type.u32);
  const source = t.params.inputSource === 'const' ? 'const' : 'non_const';
  const cases = await d.get(`division_vector${vec_size}_scalar_${source}`);
  await run(t, compoundBinary('/='), [vec_type, Type.u32], vec_type, t.params, cases);
});

g.test('remainder_scalar_vector').
specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr').
desc(
  `
Expression: x % y
`
).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize_rhs', [2, 3, 4])
).
fn(async (t) => {
  const vec_size = t.params.vectorize_rhs;
  const vec_type = Type.vec(vec_size, Type.u32);
  const source = t.params.inputSource === 'const' ? 'const' : 'non_const';
  const cases = await d.get(`remainder_scalar_vector${vec_size}_${source}`);
  await run(t, binary('%'), [Type.u32, vec_type], vec_type, t.params, cases);
});

g.test('remainder_vector_scalar').
specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr').
desc(
  `
Expression: x % y
`
).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize_lhs', [2, 3, 4])
).
fn(async (t) => {
  const vec_size = t.params.vectorize_lhs;
  const vec_type = Type.vec(vec_size, Type.u32);
  const source = t.params.inputSource === 'const' ? 'const' : 'non_const';
  const cases = await d.get(`remainder_vector${vec_size}_scalar_${source}`);
  await run(t, binary('%'), [vec_type, Type.u32], vec_type, t.params, cases);
});

g.test('remainder_vector_scalar_compound').
specURL('https://www.w3.org/TR/WGSL/#arithmetic-expr').
desc(
  `
Expression: x %= y
`
).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize_lhs', [2, 3, 4])
).
fn(async (t) => {
  const vec_size = t.params.vectorize_lhs;
  const vec_type = Type.vec(vec_size, Type.u32);
  const source = t.params.inputSource === 'const' ? 'const' : 'non_const';
  const cases = await d.get(`remainder_vector${vec_size}_scalar_${source}`);
  await run(t, compoundBinary('%='), [vec_type, Type.u32], vec_type, t.params, cases);
});