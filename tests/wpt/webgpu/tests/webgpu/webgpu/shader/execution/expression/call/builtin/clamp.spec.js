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
import { kValue } from '../../../../../util/constants.js';
import {
  TypeF32,
  TypeF16,
  TypeI32,
  TypeU32,
  TypeAbstractFloat,
} from '../../../../../util/conversion.js';
import { FP } from '../../../../../util/floating_point.js';
import { sparseF32Range, sparseF16Range, sparseF64Range } from '../../../../../util/math.js';
import { makeCaseCache } from '../../case_cache.js';
import { allInputSources, onlyConstInputSource, run } from '../../expression.js';

import { abstractBuiltin, builtin } from './builtin.js';

export const g = makeTestGroup(GPUTest);

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
  kValue.i32.positive.max,
];

export const d = makeCaseCache('clamp', {
  u32_non_const: () => {
    return generateIntegerTestCases(u32Values, TypeU32, 'non-const');
  },
  u32_const: () => {
    return generateIntegerTestCases(u32Values, TypeU32, 'const');
  },
  i32_non_const: () => {
    return generateIntegerTestCases(i32Values, TypeI32, 'non-const');
  },
  i32_const: () => {
    return generateIntegerTestCases(i32Values, TypeI32, 'const');
  },
  f32_const: () => {
    return generateFloatTestCases(sparseF32Range(), 'f32', 'const');
  },
  f32_non_const: () => {
    return generateFloatTestCases(sparseF32Range(), 'f32', 'non-const');
  },
  f16_const: () => {
    return generateFloatTestCases(sparseF16Range(), 'f16', 'const');
  },
  f16_non_const: () => {
    return generateFloatTestCases(sparseF16Range(), 'f16', 'non-const');
  },
  abstract: () => {
    return generateFloatTestCases(sparseF64Range(), 'abstract', 'const');
  },
});

/** @returns a set of clamp test cases from an ascending list of integer values */
function generateIntegerTestCases(test_values, type, stage) {
  return test_values.flatMap(low =>
    test_values.flatMap(high =>
      stage === 'const' && low > high
        ? []
        : test_values.map(e => ({
            input: [type.create(e), type.create(low), type.create(high)],
            expected: type.create(Math.min(Math.max(e, low), high)),
          }))
    )
  );
}

function generateFloatTestCases(test_values, trait, stage) {
  return test_values.flatMap(low =>
    test_values.flatMap(high =>
      stage === 'const' && low > high
        ? []
        : test_values.flatMap(e => {
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
    const cases = await d.get(t.params.inputSource === 'const' ? 'u32_const' : 'u32_non_const');
    await run(t, builtin('clamp'), [TypeU32, TypeU32, TypeU32], TypeU32, t.params, cases);
  });

g.test('i32')
  .specURL('https://www.w3.org/TR/WGSL/#integer-builtin-functions')
  .desc(`i32 tests`)
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .fn(async t => {
    const cases = await d.get(t.params.inputSource === 'const' ? 'i32_const' : 'i32_non_const');
    await run(t, builtin('clamp'), [TypeI32, TypeI32, TypeI32], TypeI32, t.params, cases);
  });

g.test('abstract_float')
  .specURL('https://www.w3.org/TR/WGSL/#float-builtin-functions')
  .desc(`abstract float tests`)
  .params(u =>
    u.combine('inputSource', onlyConstInputSource).combine('vectorize', [undefined, 2, 3, 4])
  )
  .fn(async t => {
    const cases = await d.get('abstract');
    await run(
      t,
      abstractBuiltin('clamp'),
      [TypeAbstractFloat, TypeAbstractFloat, TypeAbstractFloat],
      TypeAbstractFloat,
      t.params,
      cases
    );
  });

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
  .beforeAllSubcases(t => {
    t.selectDeviceOrSkipTestCase('shader-f16');
  })
  .fn(async t => {
    const cases = await d.get(t.params.inputSource === 'const' ? 'f16_const' : 'f16_non_const');
    await run(t, builtin('clamp'), [TypeF16, TypeF16, TypeF16], TypeF16, t.params, cases);
  });
