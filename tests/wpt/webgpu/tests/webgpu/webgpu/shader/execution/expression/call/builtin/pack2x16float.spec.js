/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Converts two floating point values to half-precision floating point numbers, and then combines them into one u32 value.
Component e[i] of the input is converted to a IEEE-754 binary16 value,
which is then placed in bits 16 × i through 16 × i + 15 of the result.
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';
import { anyOf, skipUndefined } from '../../../../../util/compare.js';
import {
  f32,
  pack2x16float,
  TypeF32,
  TypeU32,
  TypeVec,
  u32,
  vec2,
} from '../../../../../util/conversion.js';
import { cartesianProduct, fullF32Range, quantizeToF32 } from '../../../../../util/math.js';
import { makeCaseCache } from '../../case_cache.js';
import { allInputSources, run } from '../../expression.js';

import { builtin } from './builtin.js';

export const g = makeTestGroup(GPUTest);

// pack2x16float has somewhat unusual behaviour, specifically around how it is
// supposed to behave when values go OOB and when they are considered to have
// gone OOB, so has its own bespoke implementation.

/**
 * @returns a Case for `pack2x16float`
 * @param param0 first param for the case
 * @param param1 second param for the case
 * @param filter_undefined should inputs that cause an undefined expectation be
 *                         filtered out, needed for const-eval
 */
function makeCase(param0, param1, filter_undefined) {
  param0 = quantizeToF32(param0);
  param1 = quantizeToF32(param1);

  const results = pack2x16float(param0, param1);
  if (filter_undefined && results.some(r => r === undefined)) {
    return undefined;
  }

  return {
    input: [vec2(f32(param0), f32(param1))],
    expected: anyOf(
      ...results.map(r => (r === undefined ? skipUndefined(undefined) : skipUndefined(u32(r))))
    ),
  };
}

/**
 * @returns an array of Cases for `pack2x16float`
 * @param param0s array of inputs to try for the first param
 * @param param1s array of inputs to try for the second param
 * @param filter_undefined should inputs that cause an undefined expectation be
 *                         filtered out, needed for const-eval
 */
function generateCases(param0s, param1s, filter_undefined) {
  return cartesianProduct(param0s, param1s)
    .map(e => makeCase(e[0], e[1], filter_undefined))
    .filter(c => c !== undefined);
}

export const d = makeCaseCache('pack2x16float', {
  f32_const: () => {
    return generateCases(fullF32Range(), fullF32Range(), true);
  },
  f32_non_const: () => {
    return generateCases(fullF32Range(), fullF32Range(), false);
  },
});

g.test('pack')
  .specURL('https://www.w3.org/TR/WGSL/#pack-builtin-functions')
  .desc(
    `
@const fn pack2x16float(e: vec2<f32>) -> u32
`
  )
  .params(u => u.combine('inputSource', allInputSources))
  .fn(async t => {
    const cases = await d.get(t.params.inputSource === 'const' ? 'f32_const' : 'f32_non_const');
    await run(t, builtin('pack2x16float'), [TypeVec(2, TypeF32)], TypeU32, t.params, cases);
  });
