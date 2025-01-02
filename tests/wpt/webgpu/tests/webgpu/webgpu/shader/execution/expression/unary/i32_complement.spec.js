/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution Tests for the i32 bitwise complement operation
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../gpu_test.js';
import { i32, Type } from '../../../../util/conversion.js';
import { fullI32Range } from '../../../../util/math.js';
import { allInputSources, run } from '../expression.js';

import { unary } from './unary.js';

export const g = makeTestGroup(GPUTest);

g.test('i32_complement').
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
  const cases = fullI32Range().map((e) => {
    return { input: i32(e), expected: i32(~e) };
  });
  await run(t, unary('~'), [Type.i32], Type.i32, t.params, cases);
});