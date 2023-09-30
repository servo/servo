/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Execution tests for the 'select' builtin function

T is scalar, abstract numeric type, or vector
@const fn select(f: T, t: T, cond: bool) -> T
Returns t when cond is true, and f otherwise.

T is scalar or abstract numeric type
@const fn select(f: vecN<T>, t: vecN<T>, cond: vecN<bool>) -> vecN<T>
Component-wise selection. Result component i is evaluated as select(f[i],t[i],cond[i]).
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';
import {
  TypeVec,
  TypeBool,
  TypeF32,
  TypeF16,
  TypeI32,
  TypeU32,
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
} from '../../../../../util/conversion.js';
import { run, allInputSources } from '../../expression.js';

import { builtin } from './builtin.js';

export const g = makeTestGroup(GPUTest);

function makeBool(n) {
  return bool((n & 1) === 1);
}

const dataType = {
  b: {
    type: TypeBool,
    constructor: makeBool,
  },
  f: {
    type: TypeF32,
    constructor: f32,
  },
  h: {
    type: TypeF16,
    constructor: f16,
  },
  i: {
    type: TypeI32,
    constructor: i32,
  },
  u: {
    type: TypeU32,
    constructor: u32,
  },
};

g.test('scalar')
  .specURL('https://www.w3.org/TR/WGSL/#logical-builtin-functions')
  .desc(`scalar tests`)
  .params(u =>
    u
      .combine('inputSource', allInputSources)
      .combine('component', ['b', 'f', 'h', 'i', 'u'])
      .combine('overload', ['scalar', 'vec2', 'vec3', 'vec4'])
  )
  .beforeAllSubcases(t => {
    if (t.params.component === 'h') {
      t.selectDeviceOrSkipTestCase({ requiredFeatures: ['shader-f16'] });
    }
  })
  .fn(async t => {
    const componentType = dataType[t.params.component].type;
    const cons = dataType[t.params.component].constructor;

    // Create the scalar values that will be selected from, either as scalars
    // or vectors.
    //
    // Each boolean will select between c[k] and c[k+4].  Those values must
    // always compare as different.  The tricky case is boolean, where the parity
    // has to be different, i.e. c[k]-c[k+4] must be odd.
    const c = [0, 1, 2, 3, 5, 6, 7, 8].map(i => cons(i));
    // Now form vectors that will have different components from each other.
    const v2a = vec2(c[0], c[1]);
    const v2b = vec2(c[4], c[5]);
    const v3a = vec3(c[0], c[1], c[2]);
    const v3b = vec3(c[4], c[5], c[6]);
    const v4a = vec4(c[0], c[1], c[2], c[3]);
    const v4b = vec4(c[4], c[5], c[6], c[7]);

    const overloads = {
      scalar: {
        type: componentType,
        cases: [
          { input: [c[0], c[1], False], expected: c[0] },
          { input: [c[0], c[1], True], expected: c[1] },
        ],
      },
      vec2: {
        type: TypeVec(2, componentType),
        cases: [
          { input: [v2a, v2b, False], expected: v2a },
          { input: [v2a, v2b, True], expected: v2b },
        ],
      },
      vec3: {
        type: TypeVec(3, componentType),
        cases: [
          { input: [v3a, v3b, False], expected: v3a },
          { input: [v3a, v3b, True], expected: v3b },
        ],
      },
      vec4: {
        type: TypeVec(4, componentType),
        cases: [
          { input: [v4a, v4b, False], expected: v4a },
          { input: [v4a, v4b, True], expected: v4b },
        ],
      },
    };
    const overload = overloads[t.params.overload];

    await run(
      t,
      builtin('select'),
      [overload.type, overload.type, TypeBool],
      overload.type,
      t.params,
      overload.cases
    );
  });

g.test('vector')
  .specURL('https://www.w3.org/TR/WGSL/#logical-builtin-functions')
  .desc(`vector tests`)
  .params(u =>
    u
      .combine('inputSource', allInputSources)
      .combine('component', ['b', 'f', 'h', 'i', 'u'])
      .combine('overload', ['vec2', 'vec3', 'vec4'])
  )
  .beforeAllSubcases(t => {
    if (t.params.component === 'h') {
      t.selectDeviceOrSkipTestCase({ requiredFeatures: ['shader-f16'] });
    }
  })
  .fn(async t => {
    const componentType = dataType[t.params.component].type;
    const cons = dataType[t.params.component].constructor;

    // Create the scalar values that will be selected from.
    //
    // Each boolean will select between c[k] and c[k+4].  Those values must
    // always compare as different.  The tricky case is boolean, where the parity
    // has to be different, i.e. c[k]-c[k+4] must be odd.
    const c = [0, 1, 2, 3, 5, 6, 7, 8].map(i => cons(i));
    const T = True;
    const F = False;

    let tests;

    switch (t.params.overload) {
      case 'vec2': {
        const a = vec2(c[0], c[1]);
        const b = vec2(c[4], c[5]);
        tests = {
          dataType: TypeVec(2, componentType),
          boolType: TypeVec(2, TypeBool),
          cases: [
            { input: [a, b, vec2(F, F)], expected: vec2(a.x, a.y) },
            { input: [a, b, vec2(F, T)], expected: vec2(a.x, b.y) },
            { input: [a, b, vec2(T, F)], expected: vec2(b.x, a.y) },
            { input: [a, b, vec2(T, T)], expected: vec2(b.x, b.y) },
          ],
        };
        break;
      }
      case 'vec3': {
        const a = vec3(c[0], c[1], c[2]);
        const b = vec3(c[4], c[5], c[6]);
        tests = {
          dataType: TypeVec(3, componentType),
          boolType: TypeVec(3, TypeBool),
          cases: [
            { input: [a, b, vec3(F, F, F)], expected: vec3(a.x, a.y, a.z) },
            { input: [a, b, vec3(F, F, T)], expected: vec3(a.x, a.y, b.z) },
            { input: [a, b, vec3(F, T, F)], expected: vec3(a.x, b.y, a.z) },
            { input: [a, b, vec3(F, T, T)], expected: vec3(a.x, b.y, b.z) },
            { input: [a, b, vec3(T, F, F)], expected: vec3(b.x, a.y, a.z) },
            { input: [a, b, vec3(T, F, T)], expected: vec3(b.x, a.y, b.z) },
            { input: [a, b, vec3(T, T, F)], expected: vec3(b.x, b.y, a.z) },
            { input: [a, b, vec3(T, T, T)], expected: vec3(b.x, b.y, b.z) },
          ],
        };
        break;
      }
      case 'vec4': {
        const a = vec4(c[0], c[1], c[2], c[3]);
        const b = vec4(c[4], c[5], c[6], c[7]);
        tests = {
          dataType: TypeVec(4, componentType),
          boolType: TypeVec(4, TypeBool),
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
            { input: [a, b, vec4(T, T, T, T)], expected: vec4(b.x, b.y, b.z, b.w) },
          ],
        };
        break;
      }
    }

    await run(
      t,
      builtin('select'),
      [tests.dataType, tests.dataType, tests.boolType],
      tests.dataType,
      t.params,
      tests.cases
    );
  });
