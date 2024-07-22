/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution tests for the 'dot' builtin function

T is Type.abstractInt, Type.abstractFloat, i32, u32, f32, or f16
@const fn dot(e1: vecN<T>,e2: vecN<T>) -> T
Returns the dot product of e1 and e2.
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';
import { Type } from '../../../../../util/conversion.js';
import { allInputSources, onlyConstInputSource, run } from '../../expression.js';

import { abstractFloatBuiltin, abstractIntBuiltin, builtin } from './builtin.js';
import { d } from './dot.cache.js';

export const g = makeTestGroup(GPUTest);

g.test('abstract_int_vec2').
specURL('https://www.w3.org/TR/WGSL/#vector-builtin-functions').
desc(`abstract integer tests using vec2s`).
params((u) => u.combine('inputSource', onlyConstInputSource)).
fn(async (t) => {
  const cases = await d.get('abstract_int_vec2');
  await run(
    t,
    abstractIntBuiltin('dot'),
    [Type.vec(2, Type.abstractInt), Type.vec(2, Type.abstractInt)],
    Type.abstractInt,
    t.params,
    cases
  );
});

g.test('abstract_int_vec3').
specURL('https://www.w3.org/TR/WGSL/#vector-builtin-functions').
desc(`abstract integer tests using vec3s`).
params((u) => u.combine('inputSource', onlyConstInputSource)).
fn(async (t) => {
  const cases = await d.get('abstract_int_vec3');
  await run(
    t,
    abstractIntBuiltin('dot'),
    [Type.vec(3, Type.abstractInt), Type.vec(3, Type.abstractInt)],
    Type.abstractInt,
    t.params,
    cases
  );
});

g.test('abstract_int_vec4').
specURL('https://www.w3.org/TR/WGSL/#vector-builtin-functions').
desc(`abstract integer tests using vec4s`).
params((u) => u.combine('inputSource', onlyConstInputSource)).
fn(async (t) => {
  const cases = await d.get('abstract_int_vec4');
  await run(
    t,
    abstractIntBuiltin('dot'),
    [Type.vec(4, Type.abstractInt), Type.vec(4, Type.abstractInt)],
    Type.abstractInt,
    t.params,
    cases
  );
});

g.test('i32_vec2').
specURL('https://www.w3.org/TR/WGSL/#vector-builtin-functions').
desc(`i32 tests using vec2s`).
params((u) => u.combine('inputSource', allInputSources)).
fn(async (t) => {
  const cases = await d.get('i32_vec2');
  await run(t, builtin('dot'), [Type.vec2i, Type.vec2i], Type.i32, t.params, cases);
});

g.test('i32_vec3').
specURL('https://www.w3.org/TR/WGSL/#vector-builtin-functions').
desc(`i32 tests using vec3s`).
params((u) => u.combine('inputSource', allInputSources)).
fn(async (t) => {
  const cases = await d.get('i32_vec3');
  await run(t, builtin('dot'), [Type.vec3i, Type.vec3i], Type.i32, t.params, cases);
});

g.test('i32_vec4').
specURL('https://www.w3.org/TR/WGSL/#vector-builtin-functions').
desc(`i32 tests using vec4s`).
params((u) => u.combine('inputSource', allInputSources)).
fn(async (t) => {
  const cases = await d.get('i32_vec4');
  await run(t, builtin('dot'), [Type.vec4i, Type.vec4i], Type.i32, t.params, cases);
});

g.test('u32_vec2').
specURL('https://www.w3.org/TR/WGSL/#vector-builtin-functions').
desc(`u32 tests using vec2s`).
params((u) => u.combine('inputSource', allInputSources)).
fn(async (t) => {
  const cases = await d.get('u32_vec2');
  await run(t, builtin('dot'), [Type.vec2u, Type.vec2u], Type.u32, t.params, cases);
});

g.test('u32_vec3').
specURL('https://www.w3.org/TR/WGSL/#vector-builtin-functions').
desc(`u32 tests using vec3s`).
params((u) => u.combine('inputSource', allInputSources)).
fn(async (t) => {
  const cases = await d.get('u32_vec3');
  await run(t, builtin('dot'), [Type.vec3u, Type.vec3u], Type.u32, t.params, cases);
});

g.test('u32_vec4').
specURL('https://www.w3.org/TR/WGSL/#vector-builtin-functions').
desc(`u32 tests using vec4s`).
params((u) => u.combine('inputSource', allInputSources)).
fn(async (t) => {
  const cases = await d.get('u32_vec4');
  await run(t, builtin('dot'), [Type.vec4u, Type.vec4u], Type.u32, t.params, cases);
});

g.test('abstract_float_vec2').
specURL('https://www.w3.org/TR/WGSL/#vector-builtin-functions').
desc(`abstract float tests using vec2s`).
params((u) => u.combine('inputSource', onlyConstInputSource)).
fn(async (t) => {
  const cases = await d.get('abstract_float_vec2_const');
  await run(
    t,
    abstractFloatBuiltin('dot'),
    [Type.vec(2, Type.abstractFloat), Type.vec(2, Type.abstractFloat)],
    Type.abstractFloat,
    t.params,
    cases
  );
});

g.test('abstract_float_vec3').
specURL('https://www.w3.org/TR/WGSL/#vector-builtin-functions').
desc(`abstract float tests using vec3s`).
params((u) => u.combine('inputSource', onlyConstInputSource)).
fn(async (t) => {
  const cases = await d.get('abstract_float_vec3_const');
  await run(
    t,
    abstractFloatBuiltin('dot'),
    [Type.vec(3, Type.abstractFloat), Type.vec(3, Type.abstractFloat)],
    Type.abstractFloat,
    t.params,
    cases
  );
});

g.test('abstract_float_vec4').
specURL('https://www.w3.org/TR/WGSL/#vector-builtin-functions').
desc(`abstract float tests using vec4s`).
params((u) => u.combine('inputSource', onlyConstInputSource)).
fn(async (t) => {
  const cases = await d.get('abstract_float_vec4_const');
  await run(
    t,
    abstractFloatBuiltin('dot'),
    [Type.vec(4, Type.abstractFloat), Type.vec(4, Type.abstractFloat)],
    Type.abstractFloat,
    t.params,
    cases
  );
});

g.test('f32_vec2').
specURL('https://www.w3.org/TR/WGSL/#vector-builtin-functions').
desc(`f32 tests using vec2s`).
params((u) => u.combine('inputSource', allInputSources)).
fn(async (t) => {
  const cases = await d.get(
    t.params.inputSource === 'const' ? 'f32_vec2_const' : 'f32_vec2_non_const'
  );
  await run(t, builtin('dot'), [Type.vec2f, Type.vec2f], Type.f32, t.params, cases);
});

g.test('f32_vec3').
specURL('https://www.w3.org/TR/WGSL/#vector-builtin-functions').
desc(`f32 tests using vec3s`).
params((u) => u.combine('inputSource', allInputSources)).
fn(async (t) => {
  const cases = await d.get(
    t.params.inputSource === 'const' ? 'f32_vec3_const' : 'f32_vec3_non_const'
  );
  await run(t, builtin('dot'), [Type.vec3f, Type.vec3f], Type.f32, t.params, cases);
});

g.test('f32_vec4').
specURL('https://www.w3.org/TR/WGSL/#vector-builtin-functions').
desc(`f32 tests using vec4s`).
params((u) => u.combine('inputSource', allInputSources)).
fn(async (t) => {
  const cases = await d.get(
    t.params.inputSource === 'const' ? 'f32_vec4_const' : 'f32_vec4_non_const'
  );
  await run(t, builtin('dot'), [Type.vec4f, Type.vec4f], Type.f32, t.params, cases);
});

g.test('f16_vec2').
specURL('https://www.w3.org/TR/WGSL/#vector-builtin-functions').
desc(`f16 tests using vec2s`).
params((u) => u.combine('inputSource', allInputSources)).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('shader-f16');
}).
fn(async (t) => {
  const cases = await d.get(
    t.params.inputSource === 'const' ? 'f16_vec2_const' : 'f16_vec2_non_const'
  );
  await run(t, builtin('dot'), [Type.vec2h, Type.vec2h], Type.f16, t.params, cases);
});

g.test('f16_vec3').
specURL('https://www.w3.org/TR/WGSL/#vector-builtin-functions').
desc(`f16 tests using vec3s`).
params((u) => u.combine('inputSource', allInputSources)).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('shader-f16');
}).
fn(async (t) => {
  const cases = await d.get(
    t.params.inputSource === 'const' ? 'f16_vec3_const' : 'f16_vec3_non_const'
  );
  await run(t, builtin('dot'), [Type.vec3h, Type.vec3h], Type.f16, t.params, cases);
});

g.test('f16_vec4').
specURL('https://www.w3.org/TR/WGSL/#vector-builtin-functions').
desc(`f16 tests using vec4s`).
params((u) => u.combine('inputSource', allInputSources)).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('shader-f16');
}).
fn(async (t) => {
  const cases = await d.get(
    t.params.inputSource === 'const' ? 'f16_vec4_const' : 'f16_vec4_non_const'
  );
  await run(t, builtin('dot'), [Type.vec4h, Type.vec4h], Type.f16, t.params, cases);
});