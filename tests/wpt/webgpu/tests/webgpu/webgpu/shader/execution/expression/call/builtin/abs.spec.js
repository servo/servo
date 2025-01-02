/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution tests for the 'abs' builtin function

S is abstract-int, i32, or u32
T is S or vecN<S>
@const fn abs(e: T ) -> T
The absolute value of e. Component-wise when T is a vector. If e is a signed
integral scalar type and evaluates to the largest negative value, then the
result is e. If e is an unsigned integral type, then the result is e.

S is abstract-float, f32, f16
T is S or vecN<S>
@const fn abs(e: T ) -> T
Returns the absolute value of e (e.g. e with a positive sign bit).
Component-wise when T is a vector.
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';
import { kBit } from '../../../../../util/constants.js';
import { Type, i32Bits, u32Bits } from '../../../../../util/conversion.js';
import { allInputSources, onlyConstInputSource, run } from '../../expression.js';

import { d } from './abs.cache.js';
import { abstractFloatBuiltin, abstractIntBuiltin, builtin } from './builtin.js';

export const g = makeTestGroup(GPUTest);

g.test('abstract_int').
specURL('https://www.w3.org/TR/WGSL/#integer-builtin-functions').
desc(`abstract int tests`).
params((u) =>
u.
combine('inputSource', onlyConstInputSource).
combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const cases = await d.get('abstract_int');
  await run(t, abstractIntBuiltin('abs'), [Type.abstractInt], Type.abstractInt, t.params, cases);
});

g.test('u32').
specURL('https://www.w3.org/TR/WGSL/#integer-builtin-functions').
desc(`unsigned int tests`).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  await run(t, builtin('abs'), [Type.u32], Type.u32, t.params, [
  // Min and Max u32
  { input: u32Bits(kBit.u32.min), expected: u32Bits(kBit.u32.min) },
  { input: u32Bits(kBit.u32.max), expected: u32Bits(kBit.u32.max) },
  // Powers of 2: -2^i: 0 =< i =< 31
  { input: u32Bits(kBit.powTwo.to0), expected: u32Bits(kBit.powTwo.to0) },
  { input: u32Bits(kBit.powTwo.to1), expected: u32Bits(kBit.powTwo.to1) },
  { input: u32Bits(kBit.powTwo.to2), expected: u32Bits(kBit.powTwo.to2) },
  { input: u32Bits(kBit.powTwo.to3), expected: u32Bits(kBit.powTwo.to3) },
  { input: u32Bits(kBit.powTwo.to4), expected: u32Bits(kBit.powTwo.to4) },
  { input: u32Bits(kBit.powTwo.to5), expected: u32Bits(kBit.powTwo.to5) },
  { input: u32Bits(kBit.powTwo.to6), expected: u32Bits(kBit.powTwo.to6) },
  { input: u32Bits(kBit.powTwo.to7), expected: u32Bits(kBit.powTwo.to7) },
  { input: u32Bits(kBit.powTwo.to8), expected: u32Bits(kBit.powTwo.to8) },
  { input: u32Bits(kBit.powTwo.to9), expected: u32Bits(kBit.powTwo.to9) },
  { input: u32Bits(kBit.powTwo.to10), expected: u32Bits(kBit.powTwo.to10) },
  { input: u32Bits(kBit.powTwo.to11), expected: u32Bits(kBit.powTwo.to11) },
  { input: u32Bits(kBit.powTwo.to12), expected: u32Bits(kBit.powTwo.to12) },
  { input: u32Bits(kBit.powTwo.to13), expected: u32Bits(kBit.powTwo.to13) },
  { input: u32Bits(kBit.powTwo.to14), expected: u32Bits(kBit.powTwo.to14) },
  { input: u32Bits(kBit.powTwo.to15), expected: u32Bits(kBit.powTwo.to15) },
  { input: u32Bits(kBit.powTwo.to16), expected: u32Bits(kBit.powTwo.to16) },
  { input: u32Bits(kBit.powTwo.to17), expected: u32Bits(kBit.powTwo.to17) },
  { input: u32Bits(kBit.powTwo.to18), expected: u32Bits(kBit.powTwo.to18) },
  { input: u32Bits(kBit.powTwo.to19), expected: u32Bits(kBit.powTwo.to19) },
  { input: u32Bits(kBit.powTwo.to20), expected: u32Bits(kBit.powTwo.to20) },
  { input: u32Bits(kBit.powTwo.to21), expected: u32Bits(kBit.powTwo.to21) },
  { input: u32Bits(kBit.powTwo.to22), expected: u32Bits(kBit.powTwo.to22) },
  { input: u32Bits(kBit.powTwo.to23), expected: u32Bits(kBit.powTwo.to23) },
  { input: u32Bits(kBit.powTwo.to24), expected: u32Bits(kBit.powTwo.to24) },
  { input: u32Bits(kBit.powTwo.to25), expected: u32Bits(kBit.powTwo.to25) },
  { input: u32Bits(kBit.powTwo.to26), expected: u32Bits(kBit.powTwo.to26) },
  { input: u32Bits(kBit.powTwo.to27), expected: u32Bits(kBit.powTwo.to27) },
  { input: u32Bits(kBit.powTwo.to28), expected: u32Bits(kBit.powTwo.to28) },
  { input: u32Bits(kBit.powTwo.to29), expected: u32Bits(kBit.powTwo.to29) },
  { input: u32Bits(kBit.powTwo.to30), expected: u32Bits(kBit.powTwo.to30) },
  { input: u32Bits(kBit.powTwo.to31), expected: u32Bits(kBit.powTwo.to31) }]
  );
});

g.test('i32').
specURL('https://www.w3.org/TR/WGSL/#integer-builtin-functions').
desc(`signed int tests`).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  await run(t, builtin('abs'), [Type.i32], Type.i32, t.params, [
  // Min and max i32
  // If e evaluates to the largest negative value, then the result is e.
  { input: i32Bits(kBit.i32.negative.min), expected: i32Bits(kBit.i32.negative.min) },
  { input: i32Bits(kBit.i32.negative.max), expected: i32Bits(kBit.i32.positive.min) },
  { input: i32Bits(kBit.i32.positive.max), expected: i32Bits(kBit.i32.positive.max) },
  { input: i32Bits(kBit.i32.positive.min), expected: i32Bits(kBit.i32.positive.min) },
  // input: -1 * pow(2, n), n = {-31, ..., 0 }, expected: pow(2, n), n = {-31, ..., 0}]
  { input: i32Bits(kBit.negPowTwo.to0), expected: i32Bits(kBit.powTwo.to0) },
  { input: i32Bits(kBit.negPowTwo.to1), expected: i32Bits(kBit.powTwo.to1) },
  { input: i32Bits(kBit.negPowTwo.to2), expected: i32Bits(kBit.powTwo.to2) },
  { input: i32Bits(kBit.negPowTwo.to3), expected: i32Bits(kBit.powTwo.to3) },
  { input: i32Bits(kBit.negPowTwo.to4), expected: i32Bits(kBit.powTwo.to4) },
  { input: i32Bits(kBit.negPowTwo.to5), expected: i32Bits(kBit.powTwo.to5) },
  { input: i32Bits(kBit.negPowTwo.to6), expected: i32Bits(kBit.powTwo.to6) },
  { input: i32Bits(kBit.negPowTwo.to7), expected: i32Bits(kBit.powTwo.to7) },
  { input: i32Bits(kBit.negPowTwo.to8), expected: i32Bits(kBit.powTwo.to8) },
  { input: i32Bits(kBit.negPowTwo.to9), expected: i32Bits(kBit.powTwo.to9) },
  { input: i32Bits(kBit.negPowTwo.to10), expected: i32Bits(kBit.powTwo.to10) },
  { input: i32Bits(kBit.negPowTwo.to11), expected: i32Bits(kBit.powTwo.to11) },
  { input: i32Bits(kBit.negPowTwo.to12), expected: i32Bits(kBit.powTwo.to12) },
  { input: i32Bits(kBit.negPowTwo.to13), expected: i32Bits(kBit.powTwo.to13) },
  { input: i32Bits(kBit.negPowTwo.to14), expected: i32Bits(kBit.powTwo.to14) },
  { input: i32Bits(kBit.negPowTwo.to15), expected: i32Bits(kBit.powTwo.to15) },
  { input: i32Bits(kBit.negPowTwo.to16), expected: i32Bits(kBit.powTwo.to16) },
  { input: i32Bits(kBit.negPowTwo.to17), expected: i32Bits(kBit.powTwo.to17) },
  { input: i32Bits(kBit.negPowTwo.to18), expected: i32Bits(kBit.powTwo.to18) },
  { input: i32Bits(kBit.negPowTwo.to19), expected: i32Bits(kBit.powTwo.to19) },
  { input: i32Bits(kBit.negPowTwo.to20), expected: i32Bits(kBit.powTwo.to20) },
  { input: i32Bits(kBit.negPowTwo.to21), expected: i32Bits(kBit.powTwo.to21) },
  { input: i32Bits(kBit.negPowTwo.to22), expected: i32Bits(kBit.powTwo.to22) },
  { input: i32Bits(kBit.negPowTwo.to23), expected: i32Bits(kBit.powTwo.to23) },
  { input: i32Bits(kBit.negPowTwo.to24), expected: i32Bits(kBit.powTwo.to24) },
  { input: i32Bits(kBit.negPowTwo.to25), expected: i32Bits(kBit.powTwo.to25) },
  { input: i32Bits(kBit.negPowTwo.to26), expected: i32Bits(kBit.powTwo.to26) },
  { input: i32Bits(kBit.negPowTwo.to27), expected: i32Bits(kBit.powTwo.to27) },
  { input: i32Bits(kBit.negPowTwo.to28), expected: i32Bits(kBit.powTwo.to28) },
  { input: i32Bits(kBit.negPowTwo.to29), expected: i32Bits(kBit.powTwo.to29) },
  { input: i32Bits(kBit.negPowTwo.to30), expected: i32Bits(kBit.powTwo.to30) },
  { input: i32Bits(kBit.negPowTwo.to31), expected: i32Bits(kBit.powTwo.to31) }]
  );
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
  const cases = await d.get('abstract_float');
  await run(
    t,
    abstractFloatBuiltin('abs'),
    [Type.abstractFloat],
    Type.abstractFloat,
    t.params,
    cases
  );
});

g.test('f32').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(`float 32 tests`).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const cases = await d.get('f32');
  await run(t, builtin('abs'), [Type.f32], Type.f32, t.params, cases);
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
  const cases = await d.get('f16');
  await run(t, builtin('abs'), [Type.f16], Type.f16, t.params, cases);
});