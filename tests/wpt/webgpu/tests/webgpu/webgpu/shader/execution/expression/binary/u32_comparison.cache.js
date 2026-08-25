/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { bool, u32 } from '../../../../util/conversion.js';import { vectorU32Range } from '../../../../util/math.js';
import { makeCaseCache } from '../case_cache.js';

/**
 * @returns a test case for the provided left hand & right hand values and
 * expected boolean result.
 */
function makeCase(lhs, rhs, expected_answer) {
  return { input: [u32(lhs), u32(rhs)], expected: bool(expected_answer) };
}

export const d = makeCaseCache('binary/u32_comparison', {
  equals: () => vectorU32Range(2).map((v) => makeCase(v[0], v[1], v[0] === v[1])),
  not_equals: () => vectorU32Range(2).map((v) => makeCase(v[0], v[1], v[0] !== v[1])),
  less_than: () => vectorU32Range(2).map((v) => makeCase(v[0], v[1], v[0] < v[1])),
  less_equal: () => vectorU32Range(2).map((v) => makeCase(v[0], v[1], v[0] <= v[1])),
  greater_than: () => vectorU32Range(2).map((v) => makeCase(v[0], v[1], v[0] > v[1])),
  greater_equal: () => vectorU32Range(2).map((v) => makeCase(v[0], v[1], v[0] >= v[1]))
});