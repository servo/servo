/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution Tests for the u32 bitwise complement operation
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../gpu_test.js';
import { TypeU32 } from '../../../../util/conversion.js';
import { allInputSources, run } from '../expression.js';

import { d } from './u32_complement.cache.js';
import { unary } from './unary.js';

export const g = makeTestGroup(GPUTest);

g.test('u32_complement').
specURL('https://www.w3.org/TR/WGSL/#bit-expr').
desc(
  `
Expression: ~x
`
).
params((u) =>
u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const cases = await d.get('complement');
  await run(t, unary('~'), [TypeU32], TypeU32, t.params, cases);
});