/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Execution tests for the 'clamp' builtin function

S is AbstractInt, i32, or u32
T is S or vecN<S>
@const fn clamp(e: T , low: T, high: T) -> T
Returns min(max(e,low),high). Component-wise when T is a vector.

S is AbstractFloat, f32, f16
T is S or vecN<S>
@const clamp(e: T , low: T , high: T) -> T
Returns either min(max(e,low),high), or the median of the three values e, low, high.
Component-wise when T is a vector.
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';
import { kBit } from '../../../../../util/constants.js';
import {
  i32,
  i32Bits,
  TypeF32,
  TypeI32,
  TypeU32,
  u32,
  u32Bits,
} from '../../../../../util/conversion.js';
import { FP } from '../../../../../util/floating_point.js';
import { sparseF32Range } from '../../../../../util/math.js';
import { makeCaseCache } from '../../case_cache.js';
import { allInputSources, run } from '../../expression.js';

import { builtin } from './builtin.js';

export const g = makeTestGroup(GPUTest);

export const d = makeCaseCache('clamp', {
  u32: () => {
    // This array must be strictly increasing, since that ordering determines
    // the expected values.
    const test_values = [
      u32Bits(kBit.u32.min),
      u32(1),
      u32(2),
      u32(0x70000000),
      u32(0x80000000),
      u32Bits(kBit.u32.max),
    ];

    return generateIntegerTestCases(test_values);
  },
  i32: () => {
    // This array must be strictly increasing, since that ordering determines
    // the expected values.
    const test_values = [
      i32Bits(kBit.i32.negative.min),
      i32(-2),
      i32(-1),
      i32(0),
      i32(1),
      i32(2),
      i32Bits(0x70000000),
      i32Bits(kBit.i32.positive.max),
    ];

    return generateIntegerTestCases(test_values);
  },
  f32_const: () => {
    return FP.f32.generateScalarTripleToIntervalCases(
      sparseF32Range(),
      sparseF32Range(),
      sparseF32Range(),
      'finite',
      ...FP.f32.clampIntervals
    );
  },
  f32_non_const: () => {
    return FP.f32.generateScalarTripleToIntervalCases(
      sparseF32Range(),
      sparseF32Range(),
      sparseF32Range(),
      'unfiltered',
      ...FP.f32.clampIntervals
    );
  },
});

/**
 * Calculates clamp using the min-max formula.
 * clamp(e, f, g) = min(max(e, f), g)
 *
 * Operates on indices of an ascending sorted array, instead of the actual
 * values to avoid rounding issues.
 *
 * @returns the index of the clamped value
 */
function calculateMinMaxClamp(ei, fi, gi) {
  return Math.min(Math.max(ei, fi), gi);
}

/** @returns a set of clamp test cases from an ascending list of integer values */
function generateIntegerTestCases(test_values) {
  const cases = new Array();
  test_values.forEach((e, ei) => {
    test_values.forEach((f, fi) => {
      test_values.forEach((g, gi) => {
        const expected_idx = calculateMinMaxClamp(ei, fi, gi);
        const expected = test_values[expected_idx];
        cases.push({ input: [e, f, g], expected });
      });
    });
  });
  return cases;
}

g.test('abstract_int')
  .specURL('https://www.w3.org/TR/WGSL/#integer-builtin-functions')
  .desc(`abstract int tests`)
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .unimplemented();

g.test('u32')
  .specURL('https://www.w3.org/TR/WGSL/#integer-builtin-functions')
  .desc(`u32 tests`)
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .fn(async t => {
    const cases = await d.get('u32');
    await run(t, builtin('clamp'), [TypeU32, TypeU32, TypeU32], TypeU32, t.params, cases);
  });

g.test('i32')
  .specURL('https://www.w3.org/TR/WGSL/#integer-builtin-functions')
  .desc(`i32 tests`)
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .fn(async t => {
    const cases = await d.get('i32');
    await run(t, builtin('clamp'), [TypeI32, TypeI32, TypeI32], TypeI32, t.params, cases);
  });

g.test('abstract_float')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(`abstract float tests`)
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .unimplemented();

g.test('f32')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(`f32 tests`)
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .fn(async t => {
    const cases = await d.get(t.params.inputSource === 'const' ? 'f32_const' : 'f32_non_const');
    await run(t, builtin('clamp'), [TypeF32, TypeF32, TypeF32], TypeF32, t.params, cases);
  });

g.test('f16')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(`f16 tests`)
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .unimplemented();
