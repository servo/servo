/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution Tests for non-matrix f16 multiplication expression
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../gpu_test.js';
import { TypeF16, TypeVec } from '../../../../util/conversion.js';
import { allInputSources, run } from '../expression.js';

import { binary, compoundBinary } from './binary.js';
import { d } from './f16_multiplication.cache.js';

export const g = makeTestGroup(GPUTest);

g.test('scalar').
specURL('https://www.w3.org/TR/WGSL/#floating-point-evaluation').
desc(
  `
Expression: x * y, where x and y are scalars
Accuracy: Correctly rounded
`
).
params((u) => u.combine('inputSource', allInputSources)).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase({ requiredFeatures: ['shader-f16'] });
}).
fn(async (t) => {
  const cases = await d.get(
    t.params.inputSource === 'const' ? 'scalar_const' : 'scalar_non_const'
  );
  await run(t, binary('*'), [TypeF16, TypeF16], TypeF16, t.params, cases);
});

g.test('vector').
specURL('https://www.w3.org/TR/WGSL/#floating-point-evaluation').
desc(
  `
Expression: x * y, where x and y are vectors
Accuracy: Correctly rounded
`
).
params((u) => u.combine('inputSource', allInputSources).combine('vectorize', [2, 3, 4])).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase({ requiredFeatures: ['shader-f16'] });
}).
fn(async (t) => {
  const cases = await d.get(
    t.params.inputSource === 'const' ? 'scalar_const' : 'scalar_non_const' // Using vectorize to generate vector cases based on scalar cases
  );
  await run(t, binary('*'), [TypeF16, TypeF16], TypeF16, t.params, cases);
});

g.test('scalar_compound').
specURL('https://www.w3.org/TR/WGSL/#floating-point-evaluation').
desc(
  `
Expression: x *= y
Accuracy: Correctly rounded
`
).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4])
).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase({ requiredFeatures: ['shader-f16'] });
}).
fn(async (t) => {
  const cases = await d.get(
    t.params.inputSource === 'const' ? 'scalar_const' : 'scalar_non_const'
  );
  await run(t, compoundBinary('*='), [TypeF16, TypeF16], TypeF16, t.params, cases);
});

g.test('vector_scalar').
specURL('https://www.w3.org/TR/WGSL/#floating-point-evaluation').
desc(
  `
Expression: x * y, where x is a vector and y is a scalar
Accuracy: Correctly rounded
`
).
params((u) => u.combine('inputSource', allInputSources).combine('dim', [2, 3, 4])).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase({ requiredFeatures: ['shader-f16'] });
}).
fn(async (t) => {
  const dim = t.params.dim;
  const cases = await d.get(
    t.params.inputSource === 'const' ? `vec${dim}_scalar_const` : `vec${dim}_scalar_non_const`
  );
  await run(
    t,
    binary('*'),
    [TypeVec(dim, TypeF16), TypeF16],
    TypeVec(dim, TypeF16),
    t.params,
    cases
  );
});

g.test('vector_scalar_compound').
specURL('https://www.w3.org/TR/WGSL/#floating-point-evaluation').
desc(
  `
Expression: x *= y, where x is a vector and y is a scalar
Accuracy: Correctly rounded
`
).
params((u) => u.combine('inputSource', allInputSources).combine('dim', [2, 3, 4])).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase({ requiredFeatures: ['shader-f16'] });
}).
fn(async (t) => {
  const dim = t.params.dim;
  const cases = await d.get(
    t.params.inputSource === 'const' ? `vec${dim}_scalar_const` : `vec${dim}_scalar_non_const`
  );
  await run(
    t,
    compoundBinary('*='),
    [TypeVec(dim, TypeF16), TypeF16],
    TypeVec(dim, TypeF16),
    t.params,
    cases
  );
});

g.test('scalar_vector').
specURL('https://www.w3.org/TR/WGSL/#floating-point-evaluation').
desc(
  `
Expression: x * y, where x is a scalar and y is a vector
Accuracy: Correctly rounded
`
).
params((u) => u.combine('inputSource', allInputSources).combine('dim', [2, 3, 4])).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase({ requiredFeatures: ['shader-f16'] });
}).
fn(async (t) => {
  const dim = t.params.dim;
  const cases = await d.get(
    t.params.inputSource === 'const' ? `scalar_vec${dim}_const` : `scalar_vec${dim}_non_const`
  );
  await run(
    t,
    binary('*'),
    [TypeF16, TypeVec(dim, TypeF16)],
    TypeVec(dim, TypeF16),
    t.params,
    cases
  );
});