/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution tests for the 'select' builtin function

T is scalar, abstract numeric type, or vector
@const fn select(f: T, t: T, cond: bool) -> T
Returns t when cond is true, and f otherwise.

T is scalar or abstract numeric type
@const fn select(f: vecN<T>, t: vecN<T>, cond: vecN<bool>) -> vecN<T>
Component-wise selection. Result component i is evaluated as select(f[i],t[i],cond[i]).
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';
import {

  f32,
  f16,
  i32,
  u32,
  False,
  True,
  bool,
  vec2,
  vec3,
  vec4,
  abstractFloat,
  abstractInt,

  Type } from
'../../../../../util/conversion.js';

import { run, allInputSources } from '../../expression.js';

import { abstractFloatBuiltin, abstractIntBuiltin, builtin } from './builtin.js';

export const g = makeTestGroup(GPUTest);

function makeBool(n) {
  return bool((n & 1) === 1);
}



const dataType = {
  b: {
    type: Type.bool,
    scalar_builder: makeBool,
    shader_builder: builtin('select')
  },
  af: {
    type: Type.abstractFloat,
    scalar_builder: abstractFloat,
    shader_builder: abstractFloatBuiltin('select')
  },
  f: {
    type: Type.f32,
    scalar_builder: f32,
    shader_builder: builtin('select')
  },
  h: {
    type: Type.f16,
    scalar_builder: f16,
    shader_builder: builtin('select')
  },
  ai: {
    type: Type.abstractInt,
    // Only ints are used in the tests below, so the conversion to bigint will
    // be safe. If a non-int is passed in this will Error.
    scalar_builder: (v) => {
      return abstractInt(BigInt(v));
    },
    shader_builder: abstractIntBuiltin('select')
  },
  i: {
    type: Type.i32,
    scalar_builder: i32,
    shader_builder: builtin('select')
  },
  u: {
    type: Type.u32,
    scalar_builder: u32,
    shader_builder: builtin('select')
  }
};

g.test('scalar').
specURL('https://www.w3.org/TR/WGSL/#logical-builtin-functions').
desc(`scalar tests`).
params((u) =>
u.
combine('inputSource', allInputSources).
combine('component', ['b', 'af', 'f', 'h', 'ai', 'i', 'u']).
combine('overload', ['scalar', 'vec2', 'vec3', 'vec4'])
).
beforeAllSubcases((t) => {
  if (t.params.component === 'h') {
    t.selectDeviceOrSkipTestCase({ requiredFeatures: ['shader-f16'] });
  }
  t.skipIf(t.params.component === 'af' && t.params.inputSource !== 'const');
  t.skipIf(t.params.component === 'ai' && t.params.inputSource !== 'const');
}).
fn(async (t) => {
  const componentType = dataType[t.params.component].type;
  const scalar_builder = dataType[t.params.component].scalar_builder;

  // Create the scalar values that will be selected from, either as scalars
  // or vectors.
  //
  // Each boolean will select between c[k] and c[k+4].  Those values must
  // always compare as different.  The tricky case is boolean, where the parity
  // has to be different, i.e. c[k]-c[k+4] must be odd.
  const scalars = [0, 1, 2, 3, 5, 6, 7, 8].map((i) => scalar_builder(i));

  // Now form vectors that will have different components from each other.
  const v2a = vec2(scalars[0], scalars[1]);
  const v2b = vec2(scalars[4], scalars[5]);
  const v3a = vec3(scalars[0], scalars[1], scalars[2]);
  const v3b = vec3(scalars[4], scalars[5], scalars[6]);
  const v4a = vec4(scalars[0], scalars[1], scalars[2], scalars[3]);
  const v4b = vec4(scalars[4], scalars[5], scalars[6], scalars[7]);

  const overloads = {
    scalar: {
      type: componentType,
      cases: [
      { input: [scalars[0], scalars[1], False], expected: scalars[0] },
      { input: [scalars[0], scalars[1], True], expected: scalars[1] }]

    },
    vec2: {
      type: Type.vec(2, componentType),
      cases: [
      { input: [v2a, v2b, False], expected: v2a },
      { input: [v2a, v2b, True], expected: v2b }]

    },
    vec3: {
      type: Type.vec(3, componentType),
      cases: [
      { input: [v3a, v3b, False], expected: v3a },
      { input: [v3a, v3b, True], expected: v3b }]

    },
    vec4: {
      type: Type.vec(4, componentType),
      cases: [
      { input: [v4a, v4b, False], expected: v4a },
      { input: [v4a, v4b, True], expected: v4b }]

    }
  };
  const overload = overloads[t.params.overload];

  await run(
    t,
    dataType[t.params.component].shader_builder,
    [overload.type, overload.type, Type.bool],
    overload.type,
    t.params,
    overload.cases
  );
});

g.test('vector').
specURL('https://www.w3.org/TR/WGSL/#logical-builtin-functions').
desc(`vector tests`).
params((u) =>
u.
combine('inputSource', allInputSources).
combine('component', ['b', 'af', 'f', 'h', 'ai', 'i', 'u']).
combine('overload', ['vec2', 'vec3', 'vec4'])
).
beforeAllSubcases((t) => {
  if (t.params.component === 'h') {
    t.selectDeviceOrSkipTestCase({ requiredFeatures: ['shader-f16'] });
  }
  t.skipIf(t.params.component === 'af' && t.params.inputSource !== 'const');
  t.skipIf(t.params.component === 'ai' && t.params.inputSource !== 'const');
}).
fn(async (t) => {
  const componentType = dataType[t.params.component].type;
  const scalar_builder = dataType[t.params.component].scalar_builder;

  // Create the scalar values that will be selected from.
  //
  // Each boolean will select between c[k] and c[k+4].  Those values must
  // always compare as different.  The tricky case is boolean, where the parity
  // has to be different, i.e. c[k]-c[k+4] must be odd.
  const scalars = [0, 1, 2, 3, 5, 6, 7, 8].map((i) => scalar_builder(i));
  const T = True;
  const F = False;

  let tests;

  switch (t.params.overload) {
    case 'vec2':{
        const a = vec2(scalars[0], scalars[1]);
        const b = vec2(scalars[4], scalars[5]);
        tests = {
          dataType: Type.vec(2, componentType),
          boolType: Type.vec(2, Type.bool),
          cases: [
          { input: [a, b, vec2(F, F)], expected: vec2(a.x, a.y) },
          { input: [a, b, vec2(F, T)], expected: vec2(a.x, b.y) },
          { input: [a, b, vec2(T, F)], expected: vec2(b.x, a.y) },
          { input: [a, b, vec2(T, T)], expected: vec2(b.x, b.y) }]

        };
        break;
      }
    case 'vec3':{
        const a = vec3(scalars[0], scalars[1], scalars[2]);
        const b = vec3(scalars[4], scalars[5], scalars[6]);
        tests = {
          dataType: Type.vec(3, componentType),
          boolType: Type.vec(3, Type.bool),
          cases: [
          { input: [a, b, vec3(F, F, F)], expected: vec3(a.x, a.y, a.z) },
          { input: [a, b, vec3(F, F, T)], expected: vec3(a.x, a.y, b.z) },
          { input: [a, b, vec3(F, T, F)], expected: vec3(a.x, b.y, a.z) },
          { input: [a, b, vec3(F, T, T)], expected: vec3(a.x, b.y, b.z) },
          { input: [a, b, vec3(T, F, F)], expected: vec3(b.x, a.y, a.z) },
          { input: [a, b, vec3(T, F, T)], expected: vec3(b.x, a.y, b.z) },
          { input: [a, b, vec3(T, T, F)], expected: vec3(b.x, b.y, a.z) },
          { input: [a, b, vec3(T, T, T)], expected: vec3(b.x, b.y, b.z) }]

        };
        break;
      }
    case 'vec4':{
        const a = vec4(scalars[0], scalars[1], scalars[2], scalars[3]);
        const b = vec4(scalars[4], scalars[5], scalars[6], scalars[7]);
        tests = {
          dataType: Type.vec(4, componentType),
          boolType: Type.vec(4, Type.bool),
          cases: [
          { input: [a, b, vec4(F, F, F, F)], expected: vec4(a.x, a.y, a.z, a.w) },
          { input: [a, b, vec4(F, F, F, T)], expected: vec4(a.x, a.y, a.z, b.w) },
          { input: [a, b, vec4(F, F, T, F)], expected: vec4(a.x, a.y, b.z, a.w) },
          { input: [a, b, vec4(F, F, T, T)], expected: vec4(a.x, a.y, b.z, b.w) },
          { input: [a, b, vec4(F, T, F, F)], expected: vec4(a.x, b.y, a.z, a.w) },
          { input: [a, b, vec4(F, T, F, T)], expected: vec4(a.x, b.y, a.z, b.w) },
          { input: [a, b, vec4(F, T, T, F)], expected: vec4(a.x, b.y, b.z, a.w) },
          { input: [a, b, vec4(F, T, T, T)], expected: vec4(a.x, b.y, b.z, b.w) },
          { input: [a, b, vec4(T, F, F, F)], expected: vec4(b.x, a.y, a.z, a.w) },
          { input: [a, b, vec4(T, F, F, T)], expected: vec4(b.x, a.y, a.z, b.w) },
          { input: [a, b, vec4(T, F, T, F)], expected: vec4(b.x, a.y, b.z, a.w) },
          { input: [a, b, vec4(T, F, T, T)], expected: vec4(b.x, a.y, b.z, b.w) },
          { input: [a, b, vec4(T, T, F, F)], expected: vec4(b.x, b.y, a.z, a.w) },
          { input: [a, b, vec4(T, T, F, T)], expected: vec4(b.x, b.y, a.z, b.w) },
          { input: [a, b, vec4(T, T, T, F)], expected: vec4(b.x, b.y, b.z, a.w) },
          { input: [a, b, vec4(T, T, T, T)], expected: vec4(b.x, b.y, b.z, b.w) }]

        };
        break;
      }
  }

  await run(
    t,
    dataType[t.params.component].shader_builder,
    [tests.dataType, tests.dataType, tests.boolType],
    tests.dataType,
    t.params,
    tests.cases
  );
});