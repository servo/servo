/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution Tests for assignment of AbstractInts
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../gpu_test.js';
import { Type } from '../../../../util/conversion.js';
import {

  abstractIntShaderBuilder,
  basicExpressionBuilder,
  onlyConstInputSource,
  run } from
'../expression.js';

import { d } from './ai_assignment.cache.js';

function concrete_assignment() {
  return basicExpressionBuilder((value) => `${value}`);
}

function abstract_assignment() {
  return abstractIntShaderBuilder((value) => `${value}`);
}

export const g = makeTestGroup(GPUTest);

g.test('abstract').
specURL('https://www.w3.org/TR/WGSL/#abstract-types').
desc(
  `
testing that extracting abstract ints works
`
).
params((u) => u.combine('inputSource', onlyConstInputSource)).
fn(async (t) => {
  const cases = await d.get('abstract');
  await run(t, abstract_assignment(), [Type.abstractInt], Type.abstractInt, t.params, cases, 1);
});

g.test('i32').
specURL('https://www.w3.org/TR/WGSL/#i32-builtin').
desc(
  `
concretizing to i32
`
).
params((u) => u.combine('inputSource', onlyConstInputSource)).
fn(async (t) => {
  const cases = await d.get('i32');
  await run(t, concrete_assignment(), [Type.abstractInt], Type.i32, t.params, cases);
});

g.test('u32').
specURL('https://www.w3.org/TR/WGSL/#u32-builtin').
desc(
  `
concretizing to u32
`
).
params((u) => u.combine('inputSource', onlyConstInputSource)).
fn(async (t) => {
  const cases = await d.get('u32');
  await run(t, concrete_assignment(), [Type.abstractInt], Type.u32, t.params, cases);
});