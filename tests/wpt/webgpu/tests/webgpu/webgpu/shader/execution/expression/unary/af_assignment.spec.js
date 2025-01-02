/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution Tests for assignment of AbstractFloats
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../gpu_test.js';
import { Type } from '../../../../util/conversion.js';
import {

  abstractFloatShaderBuilder,
  basicExpressionBuilder,
  onlyConstInputSource,
  run } from
'../expression.js';

import { d } from './af_assignment.cache.js';

function concrete_assignment() {
  return basicExpressionBuilder((value) => `${value}`);
}

function abstract_assignment() {
  return abstractFloatShaderBuilder((value) => `${value}`);
}

export const g = makeTestGroup(GPUTest);

g.test('abstract').
specURL('https://www.w3.org/TR/WGSL/#floating-point-conversion').
desc(
  `
testing that extracting abstract floats works
`
).
params((u) => u.combine('inputSource', onlyConstInputSource)).
fn(async (t) => {
  const cases = await d.get('abstract');
  await run(
    t,
    abstract_assignment(),
    [Type.abstractFloat],
    Type.abstractFloat,
    t.params,
    cases,
    1
  );
});

g.test('f32').
specURL('https://www.w3.org/TR/WGSL/#floating-point-conversion').
desc(
  `
concretizing to f32
`
).
params((u) => u.combine('inputSource', onlyConstInputSource)).
fn(async (t) => {
  const cases = await d.get('f32');
  await run(t, concrete_assignment(), [Type.abstractFloat], Type.f32, t.params, cases);
});

g.test('f16').
specURL('https://www.w3.org/TR/WGSL/#floating-point-conversion').
desc(
  `
concretizing to f16
`
).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase({ requiredFeatures: ['shader-f16'] });
}).
params((u) => u.combine('inputSource', onlyConstInputSource)).
fn(async (t) => {
  const cases = await d.get('f16');
  await run(t, concrete_assignment(), [Type.abstractFloat], Type.f16, t.params, cases);
});