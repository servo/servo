/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution tests for the 'cross' builtin function

T is AbstractFloat, f32, or f16
@const fn cross(e1: vec3<T> ,e2: vec3<T>) -> vec3<T>
Returns the cross product of e1 and e2.
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';
import { TypeAbstractFloat, TypeF16, TypeF32, TypeVec } from '../../../../../util/conversion.js';
import { allInputSources, onlyConstInputSource, run } from '../../expression.js';

import { abstractBuiltin, builtin } from './builtin.js';
import { d } from './cross.cache.js';

export const g = makeTestGroup(GPUTest);

g.test('abstract_float').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(`abstract float tests`).
params((u) => u.combine('inputSource', onlyConstInputSource)).
fn(async (t) => {
  const cases = await d.get('abstract_const');
  await run(
    t,
    abstractBuiltin('cross'),
    [TypeVec(3, TypeAbstractFloat), TypeVec(3, TypeAbstractFloat)],
    TypeVec(3, TypeAbstractFloat),
    t.params,
    cases
  );
});

g.test('f32').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(`f32 tests`).
params((u) => u.combine('inputSource', allInputSources)).
fn(async (t) => {
  const cases = await d.get(t.params.inputSource === 'const' ? 'f32_const' : 'f32_non_const');
  await run(
    t,
    builtin('cross'),
    [TypeVec(3, TypeF32), TypeVec(3, TypeF32)],
    TypeVec(3, TypeF32),
    t.params,
    cases
  );
});

g.test('f16').
specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions').
desc(`f16 tests`).
params((u) => u.combine('inputSource', allInputSources)).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase({ requiredFeatures: ['shader-f16'] });
}).
fn(async (t) => {
  const cases = await d.get(t.params.inputSource === 'const' ? 'f16_const' : 'f16_non_const');
  await run(
    t,
    builtin('cross'),
    [TypeVec(3, TypeF16), TypeVec(3, TypeF16)],
    TypeVec(3, TypeF16),
    t.params,
    cases
  );
});