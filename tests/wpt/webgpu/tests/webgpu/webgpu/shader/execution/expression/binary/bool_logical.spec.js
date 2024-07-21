/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution Tests for the boolean binary logical expression operations
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../gpu_test.js';
import { bool, Type } from '../../../../util/conversion.js';
import { allInputSources, run } from '../expression.js';

import { binary, compoundBinary } from './binary.js';

export const g = makeTestGroup(GPUTest);

// Short circuiting vs no short circuiting is not tested here, it is covered in
// src/webgpu/shader/execution/evaluation_order.spec.ts

g.test('and').
specURL('https://www.w3.org/TR/WGSL/#logical-expr').
desc(
  `
Expression: e1 & e2
Logical "and". Component-wise when T is a vector. Evaluates both e1 and e2.
`
).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const cases = [
  { input: [bool(false), bool(false)], expected: bool(false) },
  { input: [bool(true), bool(false)], expected: bool(false) },
  { input: [bool(false), bool(true)], expected: bool(false) },
  { input: [bool(true), bool(true)], expected: bool(true) }];


  await run(t, binary('&'), [Type.bool, Type.bool], Type.bool, t.params, cases);
});

g.test('and_compound').
specURL('https://www.w3.org/TR/WGSL/#logical-expr').
desc(
  `
Expression: e1 &= e2
Logical "and". Component-wise when T is a vector. Evaluates both e1 and e2.
`
).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const cases = [
  { input: [bool(false), bool(false)], expected: bool(false) },
  { input: [bool(true), bool(false)], expected: bool(false) },
  { input: [bool(false), bool(true)], expected: bool(false) },
  { input: [bool(true), bool(true)], expected: bool(true) }];


  await run(t, compoundBinary('&='), [Type.bool, Type.bool], Type.bool, t.params, cases);
});

g.test('and_short_circuit').
specURL('https://www.w3.org/TR/WGSL/#logical-expr').
desc(
  `
Expression: e1 && e2
short_circuiting "and". Yields true if both e1 and e2 are true; evaluates e2 only if e1 is true.
`
).
params((u) => u.combine('inputSource', allInputSources)).
fn(async (t) => {
  const cases = [
  { input: [bool(false), bool(false)], expected: bool(false) },
  { input: [bool(true), bool(false)], expected: bool(false) },
  { input: [bool(false), bool(true)], expected: bool(false) },
  { input: [bool(true), bool(true)], expected: bool(true) }];


  await run(t, binary('&&'), [Type.bool, Type.bool], Type.bool, t.params, cases);
});

g.test('or').
specURL('https://www.w3.org/TR/WGSL/#logical-expr').
desc(
  `
Expression: e1 | e2
Logical "or". Component-wise when T is a vector. Evaluates both e1 and e2.
`
).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const cases = [
  { input: [bool(false), bool(false)], expected: bool(false) },
  { input: [bool(true), bool(false)], expected: bool(true) },
  { input: [bool(false), bool(true)], expected: bool(true) },
  { input: [bool(true), bool(true)], expected: bool(true) }];


  await run(t, binary('|'), [Type.bool, Type.bool], Type.bool, t.params, cases);
});

g.test('or_compound').
specURL('https://www.w3.org/TR/WGSL/#logical-expr').
desc(
  `
Expression: e1 |= e2
Logical "or". Component-wise when T is a vector. Evaluates both e1 and e2.
`
).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const cases = [
  { input: [bool(false), bool(false)], expected: bool(false) },
  { input: [bool(true), bool(false)], expected: bool(true) },
  { input: [bool(false), bool(true)], expected: bool(true) },
  { input: [bool(true), bool(true)], expected: bool(true) }];


  await run(t, compoundBinary('|='), [Type.bool, Type.bool], Type.bool, t.params, cases);
});

g.test('or_short_circuit').
specURL('https://www.w3.org/TR/WGSL/#logical-expr').
desc(
  `
Expression: e1 || e2
short_circuiting "and". Yields true if both e1 and e2 are true; evaluates e2 only if e1 is true.
`
).
params((u) => u.combine('inputSource', allInputSources)).
fn(async (t) => {
  const cases = [
  { input: [bool(false), bool(false)], expected: bool(false) },
  { input: [bool(true), bool(false)], expected: bool(true) },
  { input: [bool(false), bool(true)], expected: bool(true) },
  { input: [bool(true), bool(true)], expected: bool(true) }];


  await run(t, binary('||'), [Type.bool, Type.bool], Type.bool, t.params, cases);
});

g.test('equals').
specURL('https://www.w3.org/TR/WGSL/#logical-expr').
desc(
  `
Expression: e1 == e2
Equality. Component-wise when T is a vector.
`
).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const cases = [
  { input: [bool(false), bool(false)], expected: bool(true) },
  { input: [bool(true), bool(false)], expected: bool(false) },
  { input: [bool(false), bool(true)], expected: bool(false) },
  { input: [bool(true), bool(true)], expected: bool(true) }];


  await run(t, binary('=='), [Type.bool, Type.bool], Type.bool, t.params, cases);
});

g.test('not_equals').
specURL('https://www.w3.org/TR/WGSL/#logical-expr').
desc(
  `
Expression: e1 != e2
Equality. Component-wise when T is a vector.
`
).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const cases = [
  { input: [bool(false), bool(false)], expected: bool(false) },
  { input: [bool(true), bool(false)], expected: bool(true) },
  { input: [bool(false), bool(true)], expected: bool(true) },
  { input: [bool(true), bool(true)], expected: bool(false) }];


  await run(t, binary('!='), [Type.bool, Type.bool], Type.bool, t.params, cases);
});