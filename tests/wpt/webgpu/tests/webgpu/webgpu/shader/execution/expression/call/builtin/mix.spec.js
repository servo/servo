/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution tests for the 'mix' builtin function

S is abstract-float, f32, f16
T is S or vecN<S>
@const fn mix(e1: T, e2: T, e3: T) -> T
Returns the linear blend of e1 and e2 (e.g. e1*(1-e3)+e2*e3). Component-wise when T is a vector.

T is abstract-float, f32, or f16
T2 is vecN<T>
@const fn mix(e1: T2, e2: T2, e3: T) -> T2
Returns the component-wise linear blend of e1 and e2, using scalar blending factor e3 for each component.
Same as mix(e1,e2,T2(e3)).

`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';
import { Type } from '../../../../../util/conversion.js';
import { allInputSources, onlyConstInputSource, run } from '../../expression.js';

import { abstractFloatBuiltin, builtin } from './builtin.js';
import { d } from './mix.cache.js';

export const g = makeTestGroup(GPUTest);

g.test('abstract_float_matching').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(`abstract_float test with matching third param`).
params((u) =>
u.
combine('inputSource', onlyConstInputSource).
combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const cases = await d.get('abstract_const');
  await run(
    t,
    abstractFloatBuiltin('mix'),
    [Type.abstractFloat, Type.abstractFloat, Type.abstractFloat],
    Type.abstractFloat,
    t.params,
    cases
  );
});

g.test('abstract_float_nonmatching_vec2').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(`abstract_float tests with two vec2<abstract_float> params and scalar third param`).
params((u) => u.combine('inputSource', onlyConstInputSource)).
fn(async (t) => {
  const cases = await d.get('abstract_vec2_scalar_const');
  await run(
    t,
    abstractFloatBuiltin('mix'),
    [Type.vec(2, Type.abstractFloat), Type.vec(2, Type.abstractFloat), Type.abstractFloat],
    Type.vec(2, Type.abstractFloat),
    t.params,
    cases
  );
});

g.test('abstract_float_nonmatching_vec3').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(`abstract_float tests with two vec3<abstract_float> params and scalar third param`).
params((u) => u.combine('inputSource', onlyConstInputSource)).
fn(async (t) => {
  const cases = await d.get('abstract_vec3_scalar_const');
  await run(
    t,
    abstractFloatBuiltin('mix'),
    [Type.vec(3, Type.abstractFloat), Type.vec(3, Type.abstractFloat), Type.abstractFloat],
    Type.vec(3, Type.abstractFloat),
    t.params,
    cases
  );
});

g.test('abstract_float_nonmatching_vec4').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(`abstract_float tests with two vec4<abstract_float> params and scalar third param`).
params((u) => u.combine('inputSource', onlyConstInputSource)).
fn(async (t) => {
  const cases = await d.get('abstract_vec4_scalar_const');
  await run(
    t,
    abstractFloatBuiltin('mix'),
    [Type.vec(4, Type.abstractFloat), Type.vec(4, Type.abstractFloat), Type.abstractFloat],
    Type.vec(4, Type.abstractFloat),
    t.params,
    cases
  );
});

g.test('f32_matching').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(`f32 test with matching third param`).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const cases = await d.get(t.params.inputSource === 'const' ? 'f32_const' : 'f32_non_const');
  await run(t, builtin('mix'), [Type.f32, Type.f32, Type.f32], Type.f32, t.params, cases);
});

g.test('f32_nonmatching_vec2').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(`f32 tests with two vec2<f32> params and scalar third param`).
params((u) => u.combine('inputSource', allInputSources)).
fn(async (t) => {
  const cases = await d.get(
    t.params.inputSource === 'const' ? 'f32_vec2_scalar_const' : 'f32_vec2_scalar_non_const'
  );
  await run(t, builtin('mix'), [Type.vec2f, Type.vec2f, Type.f32], Type.vec2f, t.params, cases);
});

g.test('f32_nonmatching_vec3').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(`f32 tests with two vec3<f32> params and scalar third param`).
params((u) => u.combine('inputSource', allInputSources)).
fn(async (t) => {
  const cases = await d.get(
    t.params.inputSource === 'const' ? 'f32_vec3_scalar_const' : 'f32_vec3_scalar_non_const'
  );
  await run(t, builtin('mix'), [Type.vec3f, Type.vec3f, Type.f32], Type.vec3f, t.params, cases);
});

g.test('f32_nonmatching_vec4').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(`f32 tests with two vec4<f32> params and scalar third param`).
params((u) => u.combine('inputSource', allInputSources)).
fn(async (t) => {
  const cases = await d.get(
    t.params.inputSource === 'const' ? 'f32_vec4_scalar_const' : 'f32_vec4_scalar_non_const'
  );
  await run(t, builtin('mix'), [Type.vec4f, Type.vec4f, Type.f32], Type.vec4f, t.params, cases);
});

g.test('f16_matching').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(`f16 test with matching third param`).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4])
).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('shader-f16');
}).
fn(async (t) => {
  const cases = await d.get(t.params.inputSource === 'const' ? 'f16_const' : 'f16_non_const');
  await run(t, builtin('mix'), [Type.f16, Type.f16, Type.f16], Type.f16, t.params, cases);
});

g.test('f16_nonmatching_vec2').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(`f16 tests with two vec2<f16> params and scalar third param`).
params((u) => u.combine('inputSource', allInputSources)).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('shader-f16');
}).
fn(async (t) => {
  const cases = await d.get(
    t.params.inputSource === 'const' ? 'f16_vec2_scalar_const' : 'f16_vec2_scalar_non_const'
  );
  await run(t, builtin('mix'), [Type.vec2h, Type.vec2h, Type.f16], Type.vec2h, t.params, cases);
});

g.test('f16_nonmatching_vec3').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(`f16 tests with two vec3<f16> params and scalar third param`).
params((u) => u.combine('inputSource', allInputSources)).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('shader-f16');
}).
fn(async (t) => {
  const cases = await d.get(
    t.params.inputSource === 'const' ? 'f16_vec3_scalar_const' : 'f16_vec3_scalar_non_const'
  );
  await run(t, builtin('mix'), [Type.vec3h, Type.vec3h, Type.f16], Type.vec3h, t.params, cases);
});

g.test('f16_nonmatching_vec4').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(`f16 tests with two vec4<f16> params and scalar third param`).
params((u) => u.combine('inputSource', allInputSources)).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('shader-f16');
}).
fn(async (t) => {
  const cases = await d.get(
    t.params.inputSource === 'const' ? 'f16_vec4_scalar_const' : 'f16_vec4_scalar_non_const'
  );
  await run(t, builtin('mix'), [Type.vec4h, Type.vec4h, Type.f16], Type.vec4h, t.params, cases);
});