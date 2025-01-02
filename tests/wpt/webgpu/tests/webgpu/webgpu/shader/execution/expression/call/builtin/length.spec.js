/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution tests for the 'length' builtin function

S is abstract-float, f32, f16
T is S or vecN<S>
@const fn length(e: T ) -> f32
Returns the length of e (e.g. abs(e) if T is a scalar, or sqrt(e[0]^2 + e[1]^2 + ...) if T is a vector).
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';
import { Type } from '../../../../../util/conversion.js';
import { allInputSources, onlyConstInputSource, run } from '../../expression.js';

import { abstractFloatBuiltin, builtin } from './builtin.js';
import { d } from './length.cache.js';

export const g = makeTestGroup(GPUTest);

g.test('abstract_float').
specURL('https://www.w3.org/TR/WGSL/#numeric-builtin-functions').
desc(`abstract_float tests`).
params((u) => u.combine('inputSource', onlyConstInputSource)).
fn(async (t) => {
  const cases = await d.get('abstract');
  await run(
    t,
    abstractFloatBuiltin('length'),
    [Type.abstractFloat],
    Type.abstractFloat,
    t.params,
    cases
  );
});

g.test('abstract_float_vec2').
specURL('https://www.w3.org/TR/WGSL/#numeric-builtin-functions').
desc(`abstract float tests using vec2s`).
params((u) => u.combine('inputSource', onlyConstInputSource)).
fn(async (t) => {
  const cases = await d.get('abstract_vec2_const');
  await run(
    t,
    abstractFloatBuiltin('length'),
    [Type.vec2af],
    Type.abstractFloat,
    t.params,
    cases
  );
});

g.test('abstract_float_vec3').
specURL('https://www.w3.org/TR/WGSL/#numeric-builtin-functions').
desc(`abstract_float tests using vec3s`).
params((u) => u.combine('inputSource', onlyConstInputSource)).
fn(async (t) => {
  const cases = await d.get('abstract_vec3_const');
  await run(
    t,
    abstractFloatBuiltin('length'),
    [Type.vec3af],
    Type.abstractFloat,
    t.params,
    cases
  );
});

g.test('abstract_float_vec4').
specURL('https://www.w3.org/TR/WGSL/#numeric-builtin-functions').
desc(`abstract_float tests using vec4s`).
params((u) => u.combine('inputSource', onlyConstInputSource)).
fn(async (t) => {
  const cases = await d.get('abstract_vec4_const');
  await run(
    t,
    abstractFloatBuiltin('length'),
    [Type.vec4af],
    Type.abstractFloat,
    t.params,
    cases
  );
});

g.test('f32').
specURL('https://www.w3.org/TR/WGSL/#numeric-builtin-functions').
desc(`f32 tests`).
params((u) => u.combine('inputSource', allInputSources)).
fn(async (t) => {
  const cases = await d.get('f32');
  await run(t, builtin('length'), [Type.f32], Type.f32, t.params, cases);
});

g.test('f32_vec2').
specURL('https://www.w3.org/TR/WGSL/#numeric-builtin-functions').
desc(`f32 tests using vec2s`).
params((u) => u.combine('inputSource', allInputSources)).
fn(async (t) => {
  const cases = await d.get(
    t.params.inputSource === 'const' ? 'f32_vec2_const' : 'f32_vec2_non_const'
  );
  await run(t, builtin('length'), [Type.vec2f], Type.f32, t.params, cases);
});

g.test('f32_vec3').
specURL('https://www.w3.org/TR/WGSL/#numeric-builtin-functions').
desc(`f32 tests using vec3s`).
params((u) => u.combine('inputSource', allInputSources)).
fn(async (t) => {
  const cases = await d.get(
    t.params.inputSource === 'const' ? 'f32_vec3_const' : 'f32_vec3_non_const'
  );
  await run(t, builtin('length'), [Type.vec3f], Type.f32, t.params, cases);
});

g.test('f32_vec4').
specURL('https://www.w3.org/TR/WGSL/#numeric-builtin-functions').
desc(`f32 tests using vec4s`).
params((u) => u.combine('inputSource', allInputSources)).
fn(async (t) => {
  const cases = await d.get(
    t.params.inputSource === 'const' ? 'f32_vec4_const' : 'f32_vec4_non_const'
  );
  await run(t, builtin('length'), [Type.vec4f], Type.f32, t.params, cases);
});

g.test('f16').
specURL('https://www.w3.org/TR/WGSL/#numeric-builtin-functions').
desc(`f16 tests`).
params((u) => u.combine('inputSource', allInputSources)).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('shader-f16');
}).
fn(async (t) => {
  const cases = await d.get('f16');
  await run(t, builtin('length'), [Type.f16], Type.f16, t.params, cases);
});

g.test('f16_vec2').
specURL('https://www.w3.org/TR/WGSL/#numeric-builtin-functions').
desc(`f16 tests using vec2s`).
params((u) => u.combine('inputSource', allInputSources)).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('shader-f16');
}).
fn(async (t) => {
  const cases = await d.get(
    t.params.inputSource === 'const' ? 'f16_vec2_const' : 'f16_vec2_non_const'
  );
  await run(t, builtin('length'), [Type.vec2h], Type.f16, t.params, cases);
});

g.test('f16_vec3').
specURL('https://www.w3.org/TR/WGSL/#numeric-builtin-functions').
desc(`f16 tests using vec3s`).
params((u) => u.combine('inputSource', allInputSources)).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('shader-f16');
}).
fn(async (t) => {
  const cases = await d.get(
    t.params.inputSource === 'const' ? 'f16_vec3_const' : 'f16_vec3_non_const'
  );
  await run(t, builtin('length'), [Type.vec3h], Type.f16, t.params, cases);
});

g.test('f16_vec4').
specURL('https://www.w3.org/TR/WGSL/#numeric-builtin-functions').
desc(`f16 tests using vec4s`).
params((u) => u.combine('inputSource', allInputSources)).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('shader-f16');
}).
fn(async (t) => {
  const cases = await d.get(
    t.params.inputSource === 'const' ? 'f16_vec4_const' : 'f16_vec4_non_const'
  );
  await run(t, builtin('length'), [Type.vec4h], Type.f16, t.params, cases);
});