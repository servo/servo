/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution tests for the 'all' builtin function

S is a bool
T is S or vecN<S>
@const fn all(e: T) -> bool
Returns e if e is scalar.
Returns true if each component of e is true if e is a vector.
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';
import { False, True, Type, vec2, vec3, vec4 } from '../../../../../util/conversion.js';
import { allInputSources, run } from '../../expression.js';

import { builtin } from './builtin.js';

export const g = makeTestGroup(GPUTest);

g.test('bool').
specURL('https://www.w3.org/TR/WGSL/#logical-builtin-functions').
desc(`bool tests`).
params((u) =>
u.
combine('inputSource', allInputSources).
combine('overload', ['scalar', 'vec2', 'vec3', 'vec4'])
).
fn(async (t) => {
  const overloads = {
    scalar: {
      type: Type.bool,
      cases: [
      { input: False, expected: False },
      { input: True, expected: True }]

    },
    vec2: {
      type: Type.vec(2, Type.bool),
      cases: [
      { input: vec2(False, False), expected: False },
      { input: vec2(True, False), expected: False },
      { input: vec2(False, True), expected: False },
      { input: vec2(True, True), expected: True }]

    },
    vec3: {
      type: Type.vec(3, Type.bool),
      cases: [
      { input: vec3(False, False, False), expected: False },
      { input: vec3(True, False, False), expected: False },
      { input: vec3(False, True, False), expected: False },
      { input: vec3(True, True, False), expected: False },
      { input: vec3(False, False, True), expected: False },
      { input: vec3(True, False, True), expected: False },
      { input: vec3(False, True, True), expected: False },
      { input: vec3(True, True, True), expected: True }]

    },
    vec4: {
      type: Type.vec(4, Type.bool),
      cases: [
      { input: vec4(False, False, False, False), expected: False },
      { input: vec4(False, True, False, False), expected: False },
      { input: vec4(False, False, True, False), expected: False },
      { input: vec4(False, True, True, False), expected: False },
      { input: vec4(False, False, False, True), expected: False },
      { input: vec4(False, True, False, True), expected: False },
      { input: vec4(False, False, True, True), expected: False },
      { input: vec4(False, True, True, True), expected: False },
      { input: vec4(True, False, False, False), expected: False },
      { input: vec4(True, False, False, True), expected: False },
      { input: vec4(True, False, True, False), expected: False },
      { input: vec4(True, False, True, True), expected: False },
      { input: vec4(True, True, False, False), expected: False },
      { input: vec4(True, True, False, True), expected: False },
      { input: vec4(True, True, True, False), expected: False },
      { input: vec4(True, True, True, True), expected: True }]

    }
  };
  const overload = overloads[t.params.overload];

  await run(t, builtin('all'), [overload.type], Type.bool, t.params, overload.cases);
});