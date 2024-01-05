/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { anyOf } from '../../../../util/compare.js';import { bool, f16, f32, i32, u32 } from '../../../../util/conversion.js';import {
  fullI32Range,
  fullU32Range,
  isSubnormalNumberF16,
  isSubnormalNumberF32,
  scalarF16Range,
  scalarF32Range } from
'../../../../util/math.js';
import { makeCaseCache } from '../case_cache.js';

export const d = makeCaseCache('unary/bool_conversion', {
  bool: () => {
    return [
    { input: bool(true), expected: bool(true) },
    { input: bool(false), expected: bool(false) }];

  },
  u32: () => {
    return fullU32Range().map((u) => {
      return { input: u32(u), expected: u === 0 ? bool(false) : bool(true) };
    });
  },
  i32: () => {
    return fullI32Range().map((i) => {
      return { input: i32(i), expected: i === 0 ? bool(false) : bool(true) };
    });
  },
  f32: () => {
    return scalarF32Range().map((f) => {
      const expected = [];
      if (f !== 0) {
        expected.push(bool(true));
      }
      if (isSubnormalNumberF32(f)) {
        expected.push(bool(false));
      }
      return { input: f32(f), expected: anyOf(...expected) };
    });
  },
  f16: () => {
    return scalarF16Range().map((f) => {
      const expected = [];
      if (f !== 0) {
        expected.push(bool(true));
      }
      if (isSubnormalNumberF16(f)) {
        expected.push(bool(false));
      }
      return { input: f16(f), expected: anyOf(...expected) };
    });
  }
});