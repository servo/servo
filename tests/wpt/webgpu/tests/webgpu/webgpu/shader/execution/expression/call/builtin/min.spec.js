/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution tests for the 'min' builtin function

S is abstract-int, i32, or u32
T is S or vecN<S>
@const fn min(e1: T ,e2: T) -> T
Returns e1 if e1 is less than e2, and e2 otherwise. Component-wise when T is a vector.

S is abstract-float, f32, f16
T is S or vecN<S>
@const fn min(e1: T ,e2: T) -> T
Returns e2 if e2 is less than e1, and e1 otherwise.
If one operand is a NaN, the other is returned.
If both operands are NaNs, a NaN is returned.
Component-wise when T is a vector.
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';
import { Type, i32, u32, abstractInt } from '../../../../../util/conversion.js';
import { minBigInt } from '../../../../../util/math.js';

import { allInputSources, onlyConstInputSource, run } from '../../expression.js';

import { abstractFloatBuiltin, abstractIntBuiltin, builtin } from './builtin.js';
import { d } from './min.cache.js';

export const g = makeTestGroup(GPUTest);

/** Generate set of min test cases from list of interesting values */
function generateTestCases(values, makeCase) {
  return values.flatMap((e) => {
    return values.map((f) => {
      return makeCase(e, f);
    });
  });
}

g.test('abstract_int').
specURL('https://www.w3.org/TR/WGSL/#integer-builtin-functions').
desc(`abstract int tests`).
params((u) =>
u.
combine('inputSource', onlyConstInputSource).
combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const makeCase = (x, y) => {
    return { input: [abstractInt(x), abstractInt(y)], expected: abstractInt(minBigInt(x, y)) };
  };

  const test_values = [-0x70000000n, -2n, -1n, 0n, 1n, 2n, 0x70000000n];
  const cases = generateTestCases(test_values, makeCase);

  await run(
    t,
    abstractIntBuiltin('min'),
    [Type.abstractInt, Type.abstractInt],
    Type.abstractInt,
    t.params,
    cases
  );
});

g.test('u32').
specURL('https://www.w3.org/TR/WGSL/#integer-builtin-functions').
desc(`u32 tests`).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const makeCase = (x, y) => {
    return { input: [u32(x), u32(y)], expected: u32(Math.min(x, y)) };
  };

  const test_values = [0, 1, 2, 0x70000000, 0x80000000, 0xffffffff];
  const cases = generateTestCases(test_values, makeCase);

  await run(t, builtin('min'), [Type.u32, Type.u32], Type.u32, t.params, cases);
});

g.test('i32').
specURL('https://www.w3.org/TR/WGSL/#integer-builtin-functions').
desc(`i32 tests`).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const makeCase = (x, y) => {
    return { input: [i32(x), i32(y)], expected: i32(Math.min(x, y)) };
  };

  const test_values = [-0x70000000, -2, -1, 0, 1, 2, 0x70000000];
  const cases = generateTestCases(test_values, makeCase);

  await run(t, builtin('min'), [Type.i32, Type.i32], Type.i32, t.params, cases);
});

g.test('abstract_float').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(`abstract float tests`).
params((u) =>
u.
combine('inputSource', onlyConstInputSource).
combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const cases = await d.get('abstract');
  await run(
    t,
    abstractFloatBuiltin('min'),
    [Type.abstractFloat, Type.abstractFloat],
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
  const cases = await d.get('f32');
  await run(t, builtin('min'), [Type.f32, Type.f32], Type.f32, t.params, cases);
});

g.test('f16').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(`f16 tests`).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4])
).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase({ requiredFeatures: ['shader-f16'] });
}).
fn(async (t) => {
  const cases = await d.get('f16');
  await run(t, builtin('min'), [Type.f16, Type.f16], Type.f16, t.params, cases);
});