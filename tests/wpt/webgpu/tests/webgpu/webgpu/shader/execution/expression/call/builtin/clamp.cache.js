/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { kValue } from '../../../../../util/constants.js';import { Type } from '../../../../../util/conversion.js';import { FP } from '../../../../../util/floating_point.js';
import { maxBigInt, minBigInt } from '../../../../../util/math.js';

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


const abstractFloatValues = [
kValue.i64.negative.min,
-3n,
-2n,
-1n,
0n,
1n,
2n,
3n,
0x70000000n,
kValue.i64.positive.max];


/** @returns a set of clamp test cases from an ascending list of concrete integer values */
function generateConcreteIntegerTestCases(
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

/** @returns a set of clamp test cases from an ascending list of abstract integer values */
function generateAbstractIntegerTestCases(test_values) {
  return test_values.flatMap((low) =>
  test_values.flatMap((high) =>
  low > high ?
  [] :
  test_values.map((e) => ({
    input: [
    Type.abstractInt.create(e),
    Type.abstractInt.create(low),
    Type.abstractInt.create(high)],

    expected: Type.abstractInt.create(minBigInt(maxBigInt(e, low), high))
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
    return generateConcreteIntegerTestCases(u32Values, Type.u32, 'non_const');
  },
  u32_const: () => {
    return generateConcreteIntegerTestCases(u32Values, Type.u32, 'const');
  },
  i32_non_const: () => {
    return generateConcreteIntegerTestCases(i32Values, Type.i32, 'non_const');
  },
  i32_const: () => {
    return generateConcreteIntegerTestCases(i32Values, Type.i32, 'const');
  },
  abstract_int: () => {
    return generateAbstractIntegerTestCases(abstractFloatValues);
  },
  ...fp_cases
});