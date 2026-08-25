/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution tests for the 'trunc' builtin function

S is abstract-float, f32, f16
T is S or vecN<S>
@const fn trunc(e: T ) -> T
Returns the nearest whole number whose absolute value is less than or equal to e.
Component-wise when T is a vector.
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { AllFeaturesMaxLimitsGPUTest } from '../../../../../gpu_test.js';
import { kValue } from '../../../../../util/constants.js';
import { Type, f32, f16 } from '../../../../../util/conversion.js';
import { FP } from '../../../../../util/floating_point.js';
import {
  allInputSources,
  onlyConstInputSource,
  run,
  basicExpressionBuilder } from
'../../expression.js';

import { abstractFloatBuiltin, builtin } from './builtin.js';
import { d } from './trunc.cache.js';

export const g = makeTestGroup(AllFeaturesMaxLimitsGPUTest);

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
    abstractFloatBuiltin('trunc'),
    [Type.abstractFloat],
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
  await run(t, builtin('trunc'), [Type.f32], Type.f32, t.params, cases);
});

g.test('f16').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(`f16 tests`).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  t.skipIfDeviceDoesNotHaveFeature('shader-f16');
  const cases = await d.get('f16');
  await run(t, builtin('trunc'), [Type.f16], Type.f16, t.params, cases);
});

g.test('subnormal_division').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(`truncate subnormals made even smaller to zero`).
params((u) => u.combine('inputSource', allInputSources).combine('type', ['f32', 'f16'])).
fn(async (t) => {
  const type = t.params.type;
  if (type === 'f16') {
    t.skipIfDeviceDoesNotHaveFeature('shader-f16');
  }

  const trait = FP[type];
  const scalarBuilder = type === 'f32' ? f32 : f16;
  const constant = kValue[type];
  const cases = [
  // Smallest subnormals (closest to zero)
  {
    input: [scalarBuilder(constant.negative.subnormal.max)],
    expected: trait.toInterval(constant.negative.zero)
  },
  {
    input: [scalarBuilder(constant.positive.subnormal.min)],
    expected: trait.toInterval(constant.positive.zero)
  }];


  await run(
    t,
    // Divided by 10.0 to make subnormal even smaller and unrepresentable as subnormal, should truncate to zero
    basicExpressionBuilder((values) => `trunc(${values[0]} / 10.0)`),
    [Type[type]],
    Type[type],
    t.params,
    cases
  );
});