/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution tests for the 'sign' builtin function

S is abstract-float, Type.abstractInt, i32, f32, f16
T is S or vecN<S>
@const fn sign(e: T ) -> T
Returns the sign of e. Component-wise when T is a vector.
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';
import { Type } from '../../../../../util/conversion.js';
import { allInputSources, onlyConstInputSource, run } from '../../expression.js';

import { abstractFloatBuiltin, abstractIntBuiltin, builtin } from './builtin.js';
import { d } from './sign.cache.js';

export const g = makeTestGroup(GPUTest);

g.test('abstract_float').
specURL('https://www.w3.org/TR/WGSL/#sign-builtin').
desc(`abstract float tests`).
params((u) =>
u.
combine('inputSource', onlyConstInputSource).
combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const cases = await d.get('abstract_float');
  await run(
    t,
    abstractFloatBuiltin('sign'),
    [Type.abstractFloat],
    Type.abstractFloat,
    t.params,
    cases
  );
});

g.test('abstract_int').
specURL('https://www.w3.org/TR/WGSL/#sign-builtin').
desc(`abstract int tests`).
params((u) =>
u.
combine('inputSource', onlyConstInputSource).
combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const cases = await d.get('abstract_int');
  await run(t, abstractIntBuiltin('sign'), [Type.abstractInt], Type.abstractInt, t.params, cases);
});

g.test('i32').
specURL('https://www.w3.org/TR/WGSL/#sign-builtin').
desc(`i32 tests`).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const cases = await d.get('i32');
  await run(t, builtin('sign'), [Type.i32], Type.i32, t.params, cases);
});

g.test('f32').
specURL('https://www.w3.org/TR/WGSL/#sign-builtin').
desc(`f32 tests`).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const cases = await d.get('f32');
  await run(t, builtin('sign'), [Type.f32], Type.f32, t.params, cases);
});

g.test('f16').
specURL('https://www.w3.org/TR/WGSL/#sign-builtin').
desc(`f16 tests`).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4])
).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('shader-f16');
}).
fn(async (t) => {
  const cases = await d.get('f16');
  await run(t, builtin('sign'), [Type.f16], Type.f16, t.params, cases);
});