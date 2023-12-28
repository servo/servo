/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { kValue } from '../../../../../util/constants.js';import { TypeI32, TypeU32 } from '../../../../../util/conversion.js';import { FP } from '../../../../../util/floating_point.js';

import { makeCaseCache } from '../../case_cache.js';

const u32Values = [0, 1, 2, 3, 0x70000000, 0x80000000, kValue.u32.max];

const i32Values = [
kValue.i32.negative.min,
-3,
-2,
-1,
0,
1,
2,
3,
0x70000000,
kValue.i32.positive.max];


/** @returns a set of clamp test cases from an ascending list of integer values */
function generateIntegerTestCases(
test_values,
type,
stage)
{
  return test_values.flatMap((low) =>
  test_values.flatMap((high) =>
  stage === 'const' && low > high ?
  [] :
  test_values.map((e) => ({
    input: [type.create(e), type.create(low), type.create(high)],
    expected: type.create(Math.min(Math.max(e, low), high))
  }))
  )
  );
}

function generateFloatTestCases(
test_values,
trait,
stage)
{
  return test_values.flatMap((low) =>
  test_values.flatMap((high) =>
  stage === 'const' && low > high ?
  [] :
  test_values.flatMap((e) => {
    const c = FP[trait].makeScalarTripleToIntervalCase(
      e,
      low,
      high,
      stage === 'const' ? 'finite' : 'unfiltered',
      ...FP[trait].clampIntervals
    );
    return c === undefined ? [] : [c];
  })
  )
  );
}

// Cases: [f32|f16|abstract]_[non_]const
// abstract_non_const is empty and unused
const fp_cases = ['f32', 'f16', 'abstract'].
flatMap((trait) =>
[true, false].map((nonConst) => ({
  [`${trait}_${nonConst ? 'non_const' : 'const'}`]: () => {
    if (trait === 'abstract' && nonConst) {
      return [];
    }
    return generateFloatTestCases(
      FP[trait].sparseScalarRange(),
      trait,
      nonConst ? 'non_const' : 'const'
    );
  }
}))
).
reduce((a, b) => ({ ...a, ...b }), {});

export const d = makeCaseCache('clamp', {
  u32_non_const: () => {
    return generateIntegerTestCases(u32Values, TypeU32, 'non_const');
  },
  u32_const: () => {
    return generateIntegerTestCases(u32Values, TypeU32, 'const');
  },
  i32_non_const: () => {
    return generateIntegerTestCases(i32Values, TypeI32, 'non_const');
  },
  i32_const: () => {
    return generateIntegerTestCases(i32Values, TypeI32, 'const');
  },
  ...fp_cases
});