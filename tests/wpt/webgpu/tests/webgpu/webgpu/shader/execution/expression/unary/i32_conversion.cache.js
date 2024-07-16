/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { kValue } from '../../../../util/constants.js';import { abstractFloat,
abstractInt,
bool,
f16,
f32,
i32,
u32 } from
'../../../../util/conversion.js';
import {
  fullI32Range,
  fullU32Range,
  quantizeToF16,
  quantizeToF32,
  scalarF16Range,
  scalarF32Range,
  scalarF64Range } from
'../../../../util/math.js';
import { reinterpretU32AsI32 } from '../../../../util/reinterpret.js';
import { makeCaseCache } from '../case_cache.js';

export const d = makeCaseCache('unary/i32_conversion', {
  bool: () => {
    return [
    { input: bool(true), expected: i32(1) },
    { input: bool(false), expected: i32(0) }];

  },
  abstractInt: () => {
    return fullI32Range().map((i) => {
      return { input: abstractInt(BigInt(i)), expected: i32(i) };
    });
  },
  u32: () => {
    return fullU32Range().map((u) => {
      return { input: u32(u), expected: i32(reinterpretU32AsI32(u)) };
    });
  },
  i32: () => {
    return fullI32Range().map((i) => {
      return { input: i32(i), expected: i32(i) };
    });
  },
  abstractFloat: () => {
    return scalarF64Range().map((f) => {
      // Handles zeros and subnormals
      if (Math.abs(f) < 1.0) {
        return { input: abstractFloat(f), expected: i32(0) };
      }

      if (f <= kValue.i32.negative.min) {
        return { input: abstractFloat(f), expected: i32(kValue.i32.negative.min) };
      }

      if (f >= kValue.i32.positive.max) {
        return { input: abstractFloat(f), expected: i32(kValue.i32.positive.max) };
      }

      // All i32s are representable as f64, and both AbstractFloat and number
      // are f64 internally, so there is no need for special casing like f32 and
      // f16 below.
      return { input: abstractFloat(f), expected: i32(Math.trunc(f)) };
    });
  },
  f32: () => {
    return scalarF32Range().map((f) => {
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

      // All f32 no larger than 2^24 has a precise interger part and a fractional part, just need
      // to trunc towards 0 for the result integer.
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
  f16: () => {
    // Note that finite f16 values are always in range of i32.
    return scalarF16Range().map((f) => {
      // Handles zeros and subnormals
      if (Math.abs(f) < 1.0) {
        return { input: f16(f), expected: i32(0) };
      }

      // All f16 no larger than <= 2^12 has a precise interger part and a fractional part, just need
      // to trunc towards 0 for the result integer.
      if (Math.abs(f) <= 2 ** 12) {
        return { input: f16(f), expected: i32(Math.trunc(f)) };
      }

      // All f16s larger than 2 ** 12 are integers, so in theory one could use them directly, expect
      // that number is actually f64 internally, so they need to be quantized to f16 first.
      // Cannot just use trunc here, since that might produce a i32 value that is precise in f64,
      // but not in f16.
      return { input: f16(f), expected: i32(quantizeToF16(f)) };
    });
  }
});