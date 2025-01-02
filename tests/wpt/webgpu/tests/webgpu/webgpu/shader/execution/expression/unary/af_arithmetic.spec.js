/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution Tests for Type.abstractFloat arithmetic unary expression operations
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../gpu_test.js';
import { Type } from '../../../../util/conversion.js';
import { onlyConstInputSource, run } from '../expression.js';

import { d } from './af_arithmetic.cache.js';
import { abstractFloatUnary } from './unary.js';

export const g = makeTestGroup(GPUTest);

g.test('negation').
specURL('https://www.w3.org/TR/WGSL/#floating-point-evaluation').
desc(
  `
Expression: -x
Accuracy: Correctly rounded
`
).
params((u) =>
u.
combine('inputSource', onlyConstInputSource).
combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const cases = await d.get('negation');
  await run(
    t,
    abstractFloatUnary('-'),
    [Type.abstractFloat],
    Type.abstractFloat,
    t.params,
    cases,
    1
  );
});