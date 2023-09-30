/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Execution Tests for the u32 conversion operations
`;
import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../gpu_test.js';
import { kValue } from '../../../../util/constants.js';
import {
  bool,
  f32,
  f16,
  i32,
  reinterpretI32AsU32,
  TypeBool,
  TypeF32,
  TypeF16,
  TypeI32,
  TypeU32,
  u32,
} from '../../../../util/conversion.js';
import {
  fullF32Range,
  fullF16Range,
  fullI32Range,
  fullU32Range,
  quantizeToF32,
  quantizeToF16,
} from '../../../../util/math.js';
import { makeCaseCache } from '../case_cache.js';
import { allInputSources, run } from '../expression.js';

import { unary } from './unary.js';

export const g = makeTestGroup(GPUTest);

export const d = makeCaseCache('unary/u32_conversion', {
  bool: () => {
    return [
      { input: bool(true), expected: u32(1) },
      { input: bool(false), expected: u32(0) },
    ];
  },
  u32: () => {
    return fullU32Range().map(u => {
      return { input: u32(u), expected: u32(u) };
    });
  },
  i32: () => {
    return fullI32Range().map(i => {
      return { input: i32(i), expected: u32(reinterpretI32AsU32(i)) };
    });
  },
  f32: () => {
    return fullF32Range().map(f => {
      // Handles zeros, subnormals, and negatives
      if (f < 1.0) {
        return { input: f32(f), expected: u32(0) };
      }

      if (f >= kValue.u32.max) {
        return { input: f32(f), expected: u32(kValue.u32.max) };
      }

      // All f32 no larger than 2^24 has a precise interger part and a fractional part, just need
      // to trunc towards 0 for the result integer.
      if (f <= 2 ** 24) {
        return { input: f32(f), expected: u32(Math.floor(f)) };
      }

      // All f32s between 2 ** 24 and kValue.u32.max are integers, so in theory
      // one could use them directly, expect that number is actually f64
      // internally, so they need to be quantized to f32 first.
      // Cannot just use floor here, since that might produce a u32 value that
      // is precise in f64, but not in f32.
      return { input: f32(f), expected: u32(quantizeToF32(f)) };
    });
  },
  f16: () => {
    // Note that all positive finite f16 values are in range of u32.
    return fullF16Range().map(f => {
      // Handles zeros, subnormals, and negatives
      if (f < 1.0) {
        return { input: f16(f), expected: u32(0) };
      }

      // All f16 no larger than <= 2^12 has a precise interger part and a fractional part, just need
      // to trunc towards 0 for the result integer.
      if (f <= 2 ** 12) {
        return { input: f16(f), expected: u32(Math.trunc(f)) };
      }

      // All f16s larger than 2 ** 12 are integers, so in theory one could use them directly, expect
      // that number is actually f64 internally, so they need to be quantized to f16 first.
      // Cannot just use trunc here, since that might produce a u32 value that is precise in f64,
      // but not in f16.
      return { input: f16(f), expected: u32(quantizeToF16(f)) };
    });
  },
});

/** Generate a ShaderBuilder based on how the test case is to be vectorized */
function vectorizeToExpression(vectorize) {
  return vectorize === undefined ? unary('u32') : unary(`vec${vectorize}<u32>`);
}

g.test('bool')
  .specURL('https://www.w3.org/TR/WGSL/#value-constructor-builtin-function')
  .desc(
    `
u32(e), where e is a bool

The result is 1u if e is true and 0u otherwise
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .fn(async t => {
    const cases = await d.get('bool');
    await run(t, vectorizeToExpression(t.params.vectorize), [TypeBool], TypeU32, t.params, cases);
  });

g.test('u32')
  .specURL('https://www.w3.org/TR/WGSL/#bool-builtin')
  .desc(
    `
u32(e), where e is a u32

Identity operation
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .fn(async t => {
    const cases = await d.get('u32');
    await run(t, vectorizeToExpression(t.params.vectorize), [TypeU32], TypeU32, t.params, cases);
  });

g.test('i32')
  .specURL('https://www.w3.org/TR/WGSL/#value-constructor-builtin-function')
  .desc(
    `
u32(e), where e is a i32

Reinterpretation of bits
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .fn(async t => {
    const cases = await d.get('i32');
    await run(t, vectorizeToExpression(t.params.vectorize), [TypeI32], TypeU32, t.params, cases);
  });

g.test('f32')
  .specURL('https://www.w3.org/TR/WGSL/#value-constructor-builtin-function')
  .desc(
    `
u32(e), where e is a f32

e is converted to u32, rounding towards zero
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .fn(async t => {
    const cases = await d.get('f32');
    await run(t, vectorizeToExpression(t.params.vectorize), [TypeF32], TypeU32, t.params, cases);
  });

g.test('f16')
  .specURL('https://www.w3.org/TR/WGSL/#value-constructor-builtin-function')
  .desc(
    `
u32(e), where e is a f16

e is converted to u32, rounding towards zero
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .beforeAllSubcases(t => {
    t.selectDeviceOrSkipTestCase('shader-f16');
  })
  .fn(async t => {
    const cases = await d.get('f16');
    await run(t, vectorizeToExpression(t.params.vectorize), [TypeF16], TypeU32, t.params, cases);
  });

g.test('abstract_int')
  .specURL('https://www.w3.org/TR/WGSL/#value-constructor-builtin-function')
  .desc(
    `
u32(e), where e is an AbstractInt

Identity operation if the e can be represented in u32, otherwise it produces a shader-creation error
`
  )
  .params(u => u.combine('inputSource', allInputSources).combine('vectorize', [undefined, 2, 3, 4]))
  .unimplemented();
