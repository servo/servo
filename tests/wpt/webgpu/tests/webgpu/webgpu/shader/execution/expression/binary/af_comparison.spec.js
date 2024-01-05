/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution Tests for the AbstractFloat comparison operations
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../gpu_test.js';
import { TypeAbstractFloat, TypeBool } from '../../../../util/conversion.js';
import { allInputSources, run } from '../expression.js';

import { d } from './af_comparison.cache.js';
import { binary } from './binary.js';

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
u.
combine('inputSource', [allInputSources[0]] /* const */).
combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const cases = await d.get('equals');
  await run(t, binary('=='), [TypeAbstractFloat, TypeAbstractFloat], TypeBool, t.params, cases);
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
u.
combine('inputSource', [allInputSources[0]] /* const */).
combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const cases = await d.get('not_equals');
  await run(t, binary('!='), [TypeAbstractFloat, TypeAbstractFloat], TypeBool, t.params, cases);
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
u.
combine('inputSource', [allInputSources[0]] /* const */).
combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const cases = await d.get('less_than');
  await run(t, binary('<'), [TypeAbstractFloat, TypeAbstractFloat], TypeBool, t.params, cases);
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
u.
combine('inputSource', [allInputSources[0]] /* const */).
combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const cases = await d.get('less_equals');
  await run(t, binary('<='), [TypeAbstractFloat, TypeAbstractFloat], TypeBool, t.params, cases);
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
u.
combine('inputSource', [allInputSources[0]] /* const */).
combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const cases = await d.get('greater_than');
  await run(t, binary('>'), [TypeAbstractFloat, TypeAbstractFloat], TypeBool, t.params, cases);
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
u.
combine('inputSource', [allInputSources[0]] /* const */).
combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const cases = await d.get('greater_equals');
  await run(t, binary('>='), [TypeAbstractFloat, TypeAbstractFloat], TypeBool, t.params, cases);
});