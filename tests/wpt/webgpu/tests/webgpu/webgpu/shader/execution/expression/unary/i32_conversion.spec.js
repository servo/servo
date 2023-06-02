/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Execution Tests for the i32 conversion operations
`;
import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../gpu_test.js';
import { kValue } from '../../../../util/constants.js';
import {
  bool,
  f32,
  i32,
  reinterpretU32AsI32,
  TypeBool,
  TypeF32,
  TypeI32,
  TypeU32,
  u32,
} from '../../../../util/conversion.js';
import { fullF32Range, fullI32Range, fullU32Range, quantizeToF32 } from '../../../../util/math.js';
import { makeCaseCache } from '../case_cache.js';
import { allInputSources, run } from '../expression.js';

import { unary } from './unary.js';

export const g = makeTestGroup(GPUTest);

export const d = makeCaseCache('unary/i32_conversion', {
  bool: () => {
    return [
      { input: bool(true), expected: i32(1) },
      { input: bool(false), expected: i32(0) },
    ];
  },
  u32: () => {
    return fullU32Range().map(u => {
      return { input: u32(u), expected: i32(reinterpretU32AsI32(u)) };
    });
  },
  i32: () => {
    return fullI32Range().map(i => {
      return { input: i32(i), expected: i32(i) };
    });
  },
  f32: () => {
    return fullF32Range().map(f => {
      // Handles zeros and subnormals
      if (Math.abs(f) < 1.0) {
        return { input: f32(f), expected: i32(0) };
      }

      if (f <= kValue.i32.negative.min) {
        return { input: f32(f), expected: i32(kValue.i32.negative.min) };
      }

      if (f >= kValue.i32.positive.max) {
        return { input: f32(f), expected: i32(kValue.i32.positive.max) };
      }

      // All integers <= 2^24 are precisely representable as f32, so just need
      // to round towards 0 for the nearest integer to 0 from f.
      if (Math.abs(f) <= 2 ** 24) {
        return { input: f32(f), expected: i32(Math.trunc(f)) };
      }

      // All f32s between 2 ** 24 and kValue.i32.negative.min/.positive.max are
      // integers, so in theory one could use them directly, expect that number
      // is actually f64 internally, so they need to be quantized to f32 first.
      // Cannot just use trunc here, since that might produce a i32 value that
      // is precise in f64, but not in f32.
      return { input: f32(f), expected: i32(quantizeToF32(f)) };
    });
  },
});

/** Generate a ShaderBuilder based on how the test case is to be vectorized */
function vectorizeToExpression(vectorize) {
  return vectorize === undefined ? unary('i32') : unary(`vec${vectorize}<i32>`);
}

g.test('bool')
  .specURL('https://www.w3.org/TR/WGSL/#value-constructor-builtin-function')
  .desc(
    `
i32(e), where e is a bool

The result is 1u if e is true and 0u otherwise
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .fn(async t => {
    const cases = await d.get('bool');
    await run(t, vectorizeToExpression(t.params.vectorize), [TypeBool], TypeI32, t.params, cases);
  });

g.test('u32')
  .specURL('https://www.w3.org/TR/WGSL/#bool-builtin')
  .desc(
    `
i32(e), where e is a u32

Reinterpretation of bits
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .fn(async t => {
    const cases = await d.get('u32');
    await run(t, vectorizeToExpression(t.params.vectorize), [TypeU32], TypeI32, t.params, cases);
  });

g.test('i32')
  .specURL('https://www.w3.org/TR/WGSL/#value-constructor-builtin-function')
  .desc(
    `
i32(e), where e is a i32

Identity operation
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .fn(async t => {
    const cases = await d.get('i32');
    await run(t, vectorizeToExpression(t.params.vectorize), [TypeI32], TypeI32, t.params, cases);
  });

g.test('f32')
  .specURL('https://www.w3.org/TR/WGSL/#value-constructor-builtin-function')
  .desc(
    `
i32(e), where e is a f32

e is converted to i32, rounding towards zero
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .fn(async t => {
    const cases = await d.get('f32');
    await run(t, vectorizeToExpression(t.params.vectorize), [TypeF32], TypeI32, t.params, cases);
  });

g.test('f16')
  .specURL('https://www.w3.org/TR/WGSL/#value-constructor-builtin-function')
  .desc(
    `
i32(e), where e is a f16

e is converted to u32, rounding towards zero
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .unimplemented();
