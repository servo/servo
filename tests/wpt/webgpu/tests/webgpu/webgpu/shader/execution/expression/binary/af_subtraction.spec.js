/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution Tests for non-matrix abstract-float subtraction expression
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../gpu_test.js';
import { Type } from '../../../../util/conversion.js';
import { onlyConstInputSource, run } from '../expression.js';

import { d } from './af_subtraction.cache.js';
import { abstractFloatBinary } from './binary.js';

export const g = makeTestGroup(GPUTest);

g.test('scalar').
specURL('https://www.w3.org/TR/WGSL/#floating-point-evaluation').
desc(
  `
Expression: x - y, where x and y are scalars
Accuracy: Correctly rounded
`
).
params((u) => u.combine('inputSource', onlyConstInputSource)).
fn(async (t) => {
  const cases = await d.get('scalar');
  await run(
    t,
    abstractFloatBinary('-'),
    [Type.abstractFloat, Type.abstractFloat],
    Type.abstractFloat,
    t.params,
    cases
  );
});

g.test('vector').
specURL('https://www.w3.org/TR/WGSL/#floating-point-evaluation').
desc(
  `
Expression: x - y, where x and y are vectors
Accuracy: Correctly rounded
`
).
params((u) =>
u.combine('inputSource', onlyConstInputSource).combine('vectorize', [2, 3, 4])
).
fn(async (t) => {
  const cases = await d.get('scalar'); // Using vectorize to generate vector cases based on scalar cases
  await run(
    t,
    abstractFloatBinary('-'),
    [Type.abstractFloat, Type.abstractFloat],
    Type.abstractFloat,
    t.params,
    cases
  );
});

g.test('vector_scalar').
specURL('https://www.w3.org/TR/WGSL/#floating-point-evaluation').
desc(
  `
Expression: x - y, where x is a vector and y is a scalar
Accuracy: Correctly rounded
`
).
params((u) => u.combine('inputSource', onlyConstInputSource).combine('dim', [2, 3, 4])).
fn(async (t) => {
  const dim = t.params.dim;
  const cases = await d.get(`vec${dim}_scalar`);
  await run(
    t,
    abstractFloatBinary('-'),
    [Type.vec(dim, Type.abstractFloat), Type.abstractFloat],
    Type.vec(dim, Type.abstractFloat),
    t.params,
    cases
  );
});

g.test('scalar_vector').
specURL('https://www.w3.org/TR/WGSL/#floating-point-evaluation').
desc(
  `
Expression: x - y, where x is a scalar and y is a vector
Accuracy: Correctly rounded
`
).
params((u) => u.combine('inputSource', onlyConstInputSource).combine('dim', [2, 3, 4])).
fn(async (t) => {
  const dim = t.params.dim;
  const cases = await d.get(`scalar_vec${dim}`);
  await run(
    t,
    abstractFloatBinary('-'),
    [Type.abstractFloat, Type.vec(dim, Type.abstractFloat)],
    Type.vec(dim, Type.abstractFloat),
    t.params,
    cases
  );
});