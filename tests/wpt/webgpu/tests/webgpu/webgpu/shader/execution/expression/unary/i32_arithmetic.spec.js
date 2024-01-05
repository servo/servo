/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution Tests for the i32 arithmetic unary expression operations
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../gpu_test.js';
import { TypeI32 } from '../../../../util/conversion.js';
import { allInputSources, run } from '../expression.js';

import { d } from './i32_arithmetic.cache.js';
import { unary } from './unary.js';

export const g = makeTestGroup(GPUTest);

g.test('negation').
specURL('https://www.w3.org/TR/WGSL/#floating-point-evaluation').
desc(
  `
Expression: -x
`
).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const cases = await d.get('negation');
  await run(t, unary('-'), [TypeI32], TypeI32, t.params, cases);
});