/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution tests for the 'fwidthFine' builtin function

T is f32 or vecN<f32>
fn fwidthFine(e:T) ->T
Returns abs(dpdxFine(e)) + abs(dpdyFine(e)).
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';

import { d } from './fwidth.cache.js';
import { runFWidthTest } from './fwidth.js';

export const g = makeTestGroup(GPUTest);

const builtin = 'fwidthFine';

g.test('f32').
specURL('https://www.w3.org/TR/WGSL/#derivative-builtin-functions').
params((u) =>
u.
combine('vectorize', [undefined, 2, 3, 4]).
combine('non_uniform_discard', [false, true])
).
fn(async (t) => {
  const cases = await d.get('scalar');
  runFWidthTest(t, cases, builtin, t.params.non_uniform_discard, t.params.vectorize);
});