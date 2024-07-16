/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution Tests for the u32 comparison expressions
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../gpu_test.js';
import { Type } from '../../../../util/conversion.js';
import { allInputSources, run } from '../expression.js';

import { binary } from './binary.js';
import { d } from './u32_comparison.cache.js';

export const g = makeTestGroup(GPUTest);

g.test('equals').
specURL('https://www.w3.org/TR/WGSL/#comparison-expr').
desc(
  `
Expression: x == y
`
).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const cases = await d.get('equals');
  await run(t, binary('=='), [Type.u32, Type.u32], Type.bool, t.params, cases);
});

g.test('not_equals').
specURL('https://www.w3.org/TR/WGSL/#comparison-expr').
desc(
  `
Expression: x != y
`
).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const cases = await d.get('not_equals');
  await run(t, binary('!='), [Type.u32, Type.u32], Type.bool, t.params, cases);
});

g.test('less_than').
specURL('https://www.w3.org/TR/WGSL/#comparison-expr').
desc(
  `
Expression: x < y
`
).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const cases = await d.get('less_than');
  await run(t, binary('<'), [Type.u32, Type.u32], Type.bool, t.params, cases);
});

g.test('less_equals').
specURL('https://www.w3.org/TR/WGSL/#comparison-expr').
desc(
  `
Expression: x <= y
`
).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const cases = await d.get('less_equal');
  await run(t, binary('<='), [Type.u32, Type.u32], Type.bool, t.params, cases);
});

g.test('greater_than').
specURL('https://www.w3.org/TR/WGSL/#comparison-expr').
desc(
  `
Expression: x > y
`
).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const cases = await d.get('greater_than');
  await run(t, binary('>'), [Type.u32, Type.u32], Type.bool, t.params, cases);
});

g.test('greater_equals').
specURL('https://www.w3.org/TR/WGSL/#comparison-expr').
desc(
  `
Expression: x >= y
`
).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const cases = await d.get('greater_equal');
  await run(t, binary('>='), [Type.u32, Type.u32], Type.bool, t.params, cases);
});