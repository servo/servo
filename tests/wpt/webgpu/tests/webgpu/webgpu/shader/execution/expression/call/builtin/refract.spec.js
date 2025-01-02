/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution tests for the 'refract' builtin function

T is vecN<I>
I is abstract-float, f32, or f16
@const fn refract(e1: T ,e2: T ,e3: I ) -> T
For the incident vector e1 and surface normal e2, and the ratio of indices of
refraction e3, let k = 1.0 -e3*e3* (1.0 - dot(e2,e1) * dot(e2,e1)).
If k < 0.0, returns the refraction vector 0.0, otherwise return the refraction
vector e3*e1- (e3* dot(e2,e1) + sqrt(k)) *e2.
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';
import { Type } from '../../../../../util/conversion.js';
import { allInputSources, onlyConstInputSource, run } from '../../expression.js';

import { abstractFloatBuiltin, builtin } from './builtin.js';
import { d } from './refract.cache.js';

export const g = makeTestGroup(GPUTest);

g.test('abstract_float_vec2').
specURL('https://www.w3.org/TR/WGSL/#numeric-builtin-functions').
desc(`abstract float tests using vec2s`).
params((u) => u.combine('inputSource', onlyConstInputSource)).
fn(async (t) => {
  const cases = await d.get('abstract_vec2_const');
  await run(
    t,
    abstractFloatBuiltin('refract'),
    [Type.vec2af, Type.vec2af, Type.abstractFloat],
    Type.vec2af,
    t.params,
    cases
  );
});

g.test('abstract_float_vec3').
specURL('https://www.w3.org/TR/WGSL/#numeric-builtin-functions').
desc(`abstract float tests using vec3s`).
params((u) => u.combine('inputSource', onlyConstInputSource)).
fn(async (t) => {
  const cases = await d.get('abstract_vec3_const');
  await run(
    t,
    abstractFloatBuiltin('refract'),
    [Type.vec3af, Type.vec3af, Type.abstractFloat],
    Type.vec3af,
    t.params,
    cases
  );
});

g.test('abstract_float_vec4').
specURL('https://www.w3.org/TR/WGSL/#numeric-builtin-functions').
desc(`abstract float tests using vec4s`).
params((u) => u.combine('inputSource', onlyConstInputSource)).
fn(async (t) => {
  const cases = await d.get('abstract_vec4_const');
  await run(
    t,
    abstractFloatBuiltin('refract'),
    [Type.vec4af, Type.vec4af, Type.abstractFloat],
    Type.vec4af,
    t.params,
    cases
  );
});

g.test('f32_vec2').
specURL('https://www.w3.org/TR/WGSL/#numeric-builtin-functions').
desc(`f32 tests using vec2s`).
params((u) => u.combine('inputSource', allInputSources)).
fn(async (t) => {
  const cases = await d.get(
    t.params.inputSource === 'const' ? 'f32_vec2_const' : 'f32_vec2_non_const'
  );
  await run(
    t,
    builtin('refract'),
    [Type.vec2f, Type.vec2f, Type.f32],
    Type.vec2f,
    t.params,
    cases
  );
});

g.test('f32_vec3').
specURL('https://www.w3.org/TR/WGSL/#numeric-builtin-functions').
desc(`f32 tests using vec3s`).
params((u) => u.combine('inputSource', allInputSources)).
fn(async (t) => {
  const cases = await d.get(
    t.params.inputSource === 'const' ? 'f32_vec3_const' : 'f32_vec3_non_const'
  );
  await run(
    t,
    builtin('refract'),
    [Type.vec3f, Type.vec3f, Type.f32],
    Type.vec3f,
    t.params,
    cases
  );
});

g.test('f32_vec4').
specURL('https://www.w3.org/TR/WGSL/#numeric-builtin-functions').
desc(`f32 tests using vec4s`).
params((u) => u.combine('inputSource', allInputSources)).
fn(async (t) => {
  const cases = await d.get(
    t.params.inputSource === 'const' ? 'f32_vec4_const' : 'f32_vec4_non_const'
  );
  await run(
    t,
    builtin('refract'),
    [Type.vec4f, Type.vec4f, Type.f32],
    Type.vec4f,
    t.params,
    cases
  );
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
  await run(
    t,
    builtin('refract'),
    [Type.vec2h, Type.vec2h, Type.f16],
    Type.vec2h,
    t.params,
    cases
  );
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
  await run(
    t,
    builtin('refract'),
    [Type.vec3h, Type.vec3h, Type.f16],
    Type.vec3h,
    t.params,
    cases
  );
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
  await run(
    t,
    builtin('refract'),
    [Type.vec4h, Type.vec4h, Type.f16],
    Type.vec4h,
    t.params,
    cases
  );
});