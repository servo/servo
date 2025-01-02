/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution Tests for the Type.abstractInt bitwise complement operation
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../gpu_test.js';
import { abstractInt, Type } from '../../../../util/conversion.js';
import { fullI64Range } from '../../../../util/math.js';
import { onlyConstInputSource, run } from '../expression.js';

import { abstractIntUnary } from './unary.js';

export const g = makeTestGroup(GPUTest);

g.test('complement').
specURL('https://www.w3.org/TR/WGSL/#bit-expr').
desc(
  `
Expression: ~x
`
).
params((u) =>
u.
combine('inputSource', onlyConstInputSource).
combine('vectorize', [undefined, 2, 3, 4])
).
fn(async (t) => {
  const cases = fullI64Range().map((e) => {
    return { input: abstractInt(e), expected: abstractInt(~e) };
  });
  await run(t, abstractIntUnary('~'), [Type.abstractInt], Type.abstractInt, t.params, cases);
});