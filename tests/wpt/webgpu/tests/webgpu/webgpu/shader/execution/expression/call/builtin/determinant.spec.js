/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution tests for the 'determinant' builtin function

T is abstract-float, f32, or f16
@const determinant(e: matCxC<T> ) -> T
Returns the determinant of e.
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';
import { Type } from '../../../../../util/conversion.js';
import { allInputSources, onlyConstInputSource, run } from '../../expression.js';

import { abstractFloatBuiltin, builtin } from './builtin.js';
import { d } from './determinant.cache.js';

export const g = makeTestGroup(GPUTest);

g.test('abstract_float').
specURL('https://www.w3.org/TR/WGSL/#matrix-builtin-functions').
desc(`abstract float tests`).
params((u) => u.combine('inputSource', onlyConstInputSource).combine('dim', [2, 3, 4])).
fn(async (t) => {
  const dim = t.params.dim;
  const cases = await d.get(`abstract_mat${dim}x${dim}`);
  await run(
    t,
    abstractFloatBuiltin('determinant'),
    [Type.mat(dim, dim, Type.abstractFloat)],
    Type.abstractFloat,
    t.params,
    cases
  );
});

g.test('f32').
specURL('https://www.w3.org/TR/WGSL/#matrix-builtin-functions').
desc(`f32 tests`).
params((u) => u.combine('inputSource', allInputSources).combine('dim', [2, 3, 4])).
fn(async (t) => {
  const dim = t.params.dim;
  const cases = await d.get(
    t.params.inputSource === 'const' ?
    `f32_mat${dim}x${dim}_const` :
    `f32_mat${dim}x${dim}_non_const`
  );
  await run(t, builtin('determinant'), [Type.mat(dim, dim, Type.f32)], Type.f32, t.params, cases);
});

g.test('f16').
specURL('https://www.w3.org/TR/WGSL/#matrix-builtin-functions').
desc(`f16 tests`).
params((u) => u.combine('inputSource', allInputSources).combine('dim', [2, 3, 4])).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('shader-f16');
}).
fn(async (t) => {
  const dim = t.params.dim;
  const cases = await d.get(
    t.params.inputSource === 'const' ?
    `f16_mat${dim}x${dim}_const` :
    `f16_mat${dim}x${dim}_non_const`
  );
  await run(t, builtin('determinant'), [Type.mat(dim, dim, Type.f16)], Type.f16, t.params, cases);
});