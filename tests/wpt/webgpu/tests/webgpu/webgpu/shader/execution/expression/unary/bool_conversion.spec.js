/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution Tests for the boolean conversion operations
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../gpu_test.js';
import { Type } from '../../../../util/conversion.js';
import { allInputSources, run } from '../expression.js';

import { d } from './bool_conversion.cache.js';
import { unary } from './unary.js';

export const g = makeTestGroup(GPUTest);

/** Generate expression builder based on how the test case is to be vectorized */
function vectorizeToExpression(vectorize) {
  return vectorize === undefined ? unary('bool') : unary(`vec${vectorize}<bool>`);
}

g.test('bool').
specURL('https://www.w3.org/TR/WGSL/#value-constructor-builtin-function').
desc(
  `
bool(e), where e is a bool

Identity operation
`
).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const cases = await d.get('bool');
  await run(
    t,
    vectorizeToExpression(t.params.vectorize),
    [Type.bool],
    Type.bool,
    t.params,
    cases
  );
});

g.test('u32').
specURL('https://www.w3.org/TR/WGSL/#bool-builtin').
desc(
  `
bool(e), where e is a u32

Coercion to boolean.
The result is false if e is 0, and true otherwise.
`
).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const cases = await d.get('u32');
  await run(t, vectorizeToExpression(t.params.vectorize), [Type.u32], Type.bool, t.params, cases);
});

g.test('i32').
specURL('https://www.w3.org/TR/WGSL/#value-constructor-builtin-function').
desc(
  `
bool(e), where e is a i32

Coercion to boolean.
The result is false if e is 0, and true otherwise.
`
).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const cases = await d.get('i32');
  await run(t, vectorizeToExpression(t.params.vectorize), [Type.i32], Type.bool, t.params, cases);
});

g.test('f32').
specURL('https://www.w3.org/TR/WGSL/#value-constructor-builtin-function').
desc(
  `
bool(e), where e is a f32

Coercion to boolean.
The result is false if e is 0.0 or -0.0, and true otherwise.
`
).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const cases = await d.get('f32');
  await run(t, vectorizeToExpression(t.params.vectorize), [Type.f32], Type.bool, t.params, cases);
});

g.test('f16').
specURL('https://www.w3.org/TR/WGSL/#value-constructor-builtin-function').
desc(
  `
bool(e), where e is a f16

Coercion to boolean.
The result is false if e is 0.0 or -0.0, and true otherwise.
`
).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4])
).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('shader-f16');
}).
fn(async (t) => {
  const cases = await d.get('f16');
  await run(t, vectorizeToExpression(t.params.vectorize), [Type.f16], Type.bool, t.params, cases);
});