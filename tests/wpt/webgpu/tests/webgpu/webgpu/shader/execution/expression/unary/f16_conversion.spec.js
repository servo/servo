/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution Tests for the f32 conversion operations
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../gpu_test.js';
import { Type } from '../../../../util/conversion.js';
import { allInputSources, run, onlyConstInputSource } from '../expression.js';

import { d } from './f16_conversion.cache.js';
import { unary } from './unary.js';

export const g = makeTestGroup(GPUTest);

/** Generate a ShaderBuilder based on how the test case is to be vectorized */
function vectorizeToExpression(vectorize) {
  return vectorize === undefined ? unary('f16') : unary(`vec${vectorize}<f16>`);
}

/** Generate a ShaderBuilder for a matrix of the provided dimensions */
function matrixExperession(cols, rows) {
  return unary(`mat${cols}x${rows}<f16>`);
}

g.test('bool').
specURL('https://www.w3.org/TR/WGSL/#value-constructor-builtin-function').
desc(
  `
f16(e), where e is a bool

The result is 1.0 if e is true and 0.0 otherwise
`
).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4])
).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase({ requiredFeatures: ['shader-f16'] });
}).
fn(async (t) => {
  const cases = await d.get('bool');
  await run(t, vectorizeToExpression(t.params.vectorize), [Type.bool], Type.f16, t.params, cases);
});

g.test('u32').
specURL('https://www.w3.org/TR/WGSL/#bool-builtin').
desc(
  `
f16(e), where e is a u32

Converted to f16, +/-Inf if out of range
`
).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4])
).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase({ requiredFeatures: ['shader-f16'] });
}).
fn(async (t) => {
  const cases = await d.get(t.params.inputSource === 'const' ? 'u32_const' : 'u32_non_const');
  await run(t, vectorizeToExpression(t.params.vectorize), [Type.u32], Type.f16, t.params, cases);
});

g.test('i32').
specURL('https://www.w3.org/TR/WGSL/#value-constructor-builtin-function').
desc(
  `
f16(e), where e is a i32

Converted to f16, +/-Inf if out of range
`
).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4])
).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase({ requiredFeatures: ['shader-f16'] });
}).
fn(async (t) => {
  const cases = await d.get(t.params.inputSource === 'const' ? 'i32_const' : 'i32_non_const');
  await run(t, vectorizeToExpression(t.params.vectorize), [Type.i32], Type.f16, t.params, cases);
});

g.test('abstract_int').
specURL('https://www.w3.org/TR/WGSL/#value-constructor-builtin-function').
desc(
  `
f16(e), where e is an AbstractInt

Converted to f16, +/-Inf if out of range
`
).
params((u) =>
u.
combine('inputSource', onlyConstInputSource).
combine('vectorize', [undefined, 2, 3, 4])
).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase({ requiredFeatures: ['shader-f16'] });
}).
fn(async (t) => {
  const cases = await d.get('abstract_int');
  await run(
    t,
    vectorizeToExpression(t.params.vectorize),
    [Type.abstractInt],
    Type.f16,
    t.params,
    cases
  );
});

g.test('f32').
specURL('https://www.w3.org/TR/WGSL/#value-constructor-builtin-function').
desc(
  `
f16(e), where e is a f32

Correctly rounded to f16
`
).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4])
).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase({ requiredFeatures: ['shader-f16'] });
}).
fn(async (t) => {
  const cases = await d.get(t.params.inputSource === 'const' ? 'f32_const' : 'f32_non_const');
  await run(t, vectorizeToExpression(t.params.vectorize), [Type.f32], Type.f16, t.params, cases);
});

g.test('f32_mat').
specURL('https://www.w3.org/TR/WGSL/#matrix-builtin-functions').
desc(`f32 matrix to f16 matrix tests`).
params((u) =>
u.
combine('inputSource', allInputSources).
combine('cols', [2, 3, 4]).
combine('rows', [2, 3, 4])
).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase({ requiredFeatures: ['shader-f16'] });
}).
fn(async (t) => {
  const cols = t.params.cols;
  const rows = t.params.rows;
  const cases = await d.get(
    t.params.inputSource === 'const' ?
    `f32_mat${cols}x${rows}_const` :
    `f32_mat${cols}x${rows}_non_const`
  );
  await run(
    t,
    matrixExperession(cols, rows),
    [Type.mat(cols, rows, Type.f32)],
    Type.mat(cols, rows, Type.f16),
    t.params,
    cases
  );
});

g.test('f16').
specURL('https://www.w3.org/TR/WGSL/#value-constructor-builtin-function').
desc(
  `
  f16(e), where e is a f16

  Identical.
  `
).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4])
).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase({ requiredFeatures: ['shader-f16'] });
}).
fn(async (t) => {
  const cases = await d.get('f16');
  await run(t, vectorizeToExpression(t.params.vectorize), [Type.f16], Type.f16, t.params, cases);
});

g.test('f16_mat').
specURL('https://www.w3.org/TR/WGSL/#matrix-builtin-functions').
desc(`f16 matrix to f16 matrix tests, expected identical`).
params((u) =>
u.
combine('inputSource', allInputSources).
combine('cols', [2, 3, 4]).
combine('rows', [2, 3, 4])
).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase({ requiredFeatures: ['shader-f16'] });
}).
fn(async (t) => {
  const cols = t.params.cols;
  const rows = t.params.rows;
  const cases = await d.get(
    t.params.inputSource === 'const' ?
    `f16_mat${cols}x${rows}_const` :
    `f16_mat${cols}x${rows}_non_const`
  );
  await run(
    t,
    matrixExperession(cols, rows),
    [Type.mat(cols, rows, Type.f16)],
    Type.mat(cols, rows, Type.f16),
    t.params,
    cases
  );
});

g.test('abstract_float').
specURL('https://www.w3.org/TR/WGSL/#value-constructor-builtin-function').
desc(
  `
f16(e), where e is an AbstractFloat

Correctly rounded to f16
`
).
params((u) =>
u.
combine('inputSource', onlyConstInputSource).
combine('vectorize', [undefined, 2, 3, 4])
).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase({ requiredFeatures: ['shader-f16'] });
}).
fn(async (t) => {
  const cases = await d.get('abstract_float');
  await run(
    t,
    vectorizeToExpression(t.params.vectorize),
    [Type.abstractFloat],
    Type.f16,
    t.params,
    cases
  );
});

g.test('abstract_float_mat').
specURL('https://www.w3.org/TR/WGSL/#matrix-builtin-functions').
desc(`AbstractFloat matrix to f16 matrix tests`).
params((u) =>
u.
combine('inputSource', onlyConstInputSource).
combine('cols', [2, 3, 4]).
combine('rows', [2, 3, 4])
).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase({ requiredFeatures: ['shader-f16'] });
}).
fn(async (t) => {
  const cols = t.params.cols;
  const rows = t.params.rows;
  const cases = await d.get(`abstract_float_mat${cols}x${rows}`);
  await run(
    t,
    matrixExperession(cols, rows),
    [Type.mat(cols, rows, Type.abstractFloat)],
    Type.mat(cols, rows, Type.f16),
    t.params,
    cases
  );
});