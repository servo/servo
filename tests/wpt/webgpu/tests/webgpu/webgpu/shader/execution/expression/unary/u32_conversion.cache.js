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
import { reinterpretI32AsU32 } from '../../../../util/reinterpret.js';
import { makeCaseCache } from '../case_cache.js';

export const d = makeCaseCache('unary/u32_conversion', {
  bool: () => {
    return [
    { input: bool(true), expected: u32(1) },
    { input: bool(false), expected: u32(0) }];

  },
  abstractInt: () => {
    return fullU32Range().map((u) => {
      return { input: abstractInt(BigInt(u)), expected: u32(u) };
    });
  },
  u32: () => {
    return fullU32Range().map((u) => {
      return { input: u32(u), expected: u32(u) };
    });
  },
  i32: () => {
    return fullI32Range().map((i) => {
      return { input: i32(i), expected: u32(reinterpretI32AsU32(i)) };
    });
  },
  abstractFloat: () => {
    return [...scalarF64Range(), -1].map((f) => {
      // Handles zeros, subnormals, and negatives
      if (f < 1.0) {
        return { input: abstractFloat(f), expected: u32(0) };
      }

      if (f >= kValue.u32.max) {
        return { input: abstractFloat(f), expected: u32(kValue.u32.max) };
      }

      // All u32s are representable as f64s and number is a f64 internally, so
      // no need for special handling like is done for f32 and f16 below.
      return { input: abstractFloat(f), expected: u32(Math.floor(f)) };
    });
  },
  f32: () => {
    return scalarF32Range().map((f) => {
      // Handles zeros, subnormals, and negatives
      if (f < 1.0) {
        return { input: f32(f), expected: u32(0) };
      }

      if (f >= kValue.u32.max) {
        return { input: f32(f), expected: u32(kValue.u32.max) };
      }

      // All f32 no larger than 2^24 has a precise integer part and a fractional
      // part, just need to trunc towards 0 for the result integer.
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
    return scalarF16Range().map((f) => {
      // Handles zeros, subnormals, and negatives
      if (f < 1.0) {
        return { input: f16(f), expected: u32(0) };
      }

      // All f16 no larger than <= 2^12 has a precise integer part and a
      // fractional part, just need to trunc towards 0 for the result integer.
      if (f <= 2 ** 12) {
        return { input: f16(f), expected: u32(Math.trunc(f)) };
      }

      // All f16s larger than 2 ** 12 are integers, so in theory one could use
      // them directly, expect that number is actually f64 internally, so they
      // need to be quantized to f16 first.Cannot just use trunc here, since
      // that might produce a u32 value that is precise in f64, but not in f16.
      return { input: f16(f), expected: u32(quantizeToF16(f)) };
    });
  }
});