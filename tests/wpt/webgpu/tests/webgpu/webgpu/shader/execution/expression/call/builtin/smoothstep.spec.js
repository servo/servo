/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution tests for the 'smoothstep' builtin function

S is abstract-float, f32, f16
T is S or vecN<S>
@const fn smoothstep(low: T , high: T , x: T ) -> T
Returns the smooth Hermite interpolation between 0 and 1.
Component-wise when T is a vector.
For scalar T, the result is t * t * (3.0 - 2.0 * t), where t = clamp((x - low) / (high - low), 0.0, 1.0).

If low >= high:
* It is a shader-creation error if low and high are const-expressions.
* It is a pipeline-creation error if low and high are override-expressions.
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';
import { Type } from '../../../../../util/conversion.js';

import { allInputSources, onlyConstInputSource, run } from '../../expression.js';

import { abstractFloatBuiltin, builtin } from './builtin.js';
import { d } from './smoothstep.cache.js';

export const g = makeTestGroup(GPUTest);

// Returns true if `c` is valid for a const evaluation of smoothstep.
function validForConst(c) {
  const low = c.input[0];
  const high = c.input[1];
  return low.value < high.value;
}

g.test('abstract_float').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(`abstract float tests`).
params((u) =>
u.
combine('inputSource', onlyConstInputSource).
combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const cases = (await d.get('abstract_const')).filter((c) => validForConst(c));
  await run(
    t,
    abstractFloatBuiltin('smoothstep'),
    [Type.abstractFloat, Type.abstractFloat, Type.abstractFloat],
    Type.abstractFloat,
    t.params,
    cases
  );
});

g.test('f32').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(`f32 tests`).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const cases = await d.get(t.params.inputSource === 'const' ? 'f32_const' : 'f32_non_const');
  const validCases = cases.filter((c) => t.params.inputSource !== 'const' || validForConst(c));
  await run(
    t,
    builtin('smoothstep'),
    [Type.f32, Type.f32, Type.f32],
    Type.f32,
    t.params,
    validCases
  );
});

g.test('f16').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(`f16 tests`).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4])
).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('shader-f16');
}).
fn(async (t) => {
  const cases = await d.get(t.params.inputSource === 'const' ? 'f16_const' : 'f16_non_const');
  const validCases = cases.filter((c) => t.params.inputSource !== 'const' || validForConst(c));
  await run(
    t,
    builtin('smoothstep'),
    [Type.f16, Type.f16, Type.f16],
    Type.f16,
    t.params,
    validCases
  );
});