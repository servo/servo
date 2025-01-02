/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution Tests for the f32 comparison operations
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../gpu_test.js';
import { Type } from '../../../../util/conversion.js';
import { allInputSources, run } from '../expression.js';

import { binary } from './binary.js';
import { d } from './f32_comparison.cache.js';

export const g = makeTestGroup(GPUTest);

g.test('equals').
specURL('https://www.w3.org/TR/WGSL/#comparison-expr').
desc(
  `
Expression: x == y
Accuracy: Correct result
`
).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const cases = await d.get(
    t.params.inputSource === 'const' ? 'equals_const' : 'equals_non_const'
  );
  await run(t, binary('=='), [Type.f32, Type.f32], Type.bool, t.params, cases);
});

g.test('not_equals').
specURL('https://www.w3.org/TR/WGSL/#comparison-expr').
desc(
  `
Expression: x != y
Accuracy: Correct result
`
).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const cases = await d.get(
    t.params.inputSource === 'const' ? 'not_equals_const' : 'not_equals_non_const'
  );
  await run(t, binary('!='), [Type.f32, Type.f32], Type.bool, t.params, cases);
});

g.test('less_than').
specURL('https://www.w3.org/TR/WGSL/#comparison-expr').
desc(
  `
Expression: x < y
Accuracy: Correct result
`
).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const cases = await d.get(
    t.params.inputSource === 'const' ? 'less_than_const' : 'less_than_non_const'
  );
  await run(t, binary('<'), [Type.f32, Type.f32], Type.bool, t.params, cases);
});

g.test('less_equals').
specURL('https://www.w3.org/TR/WGSL/#comparison-expr').
desc(
  `
Expression: x <= y
Accuracy: Correct result
`
).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const cases = await d.get(
    t.params.inputSource === 'const' ? 'less_equals_const' : 'less_equals_non_const'
  );
  await run(t, binary('<='), [Type.f32, Type.f32], Type.bool, t.params, cases);
});

g.test('greater_than').
specURL('https://www.w3.org/TR/WGSL/#comparison-expr').
desc(
  `
Expression: x > y
Accuracy: Correct result
`
).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const cases = await d.get(
    t.params.inputSource === 'const' ? 'greater_than_const' : 'greater_than_non_const'
  );
  await run(t, binary('>'), [Type.f32, Type.f32], Type.bool, t.params, cases);
});

g.test('greater_equals').
specURL('https://www.w3.org/TR/WGSL/#comparison-expr').
desc(
  `
Expression: x >= y
Accuracy: Correct result
`
).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const cases = await d.get(
    t.params.inputSource === 'const' ? 'greater_equals_const' : 'greater_equals_non_const'
  );
  await run(t, binary('>='), [Type.f32, Type.f32], Type.bool, t.params, cases);
});