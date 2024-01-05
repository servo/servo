/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { bool, f16, i32, u32 } from '../../../../util/conversion.js';import { FP, FPInterval } from '../../../../util/floating_point.js';import {
  fullI32Range,
  fullU32Range,
  scalarF16Range,
  scalarF32Range,
  sparseMatrixF16Range,
  sparseMatrixF32Range } from
'../../../../util/math.js';
import { makeCaseCache } from '../case_cache.js';

const f16FiniteRangeInterval = new FPInterval(
  'f32',
  FP.f16.constants().negative.min,
  FP.f16.constants().positive.max
);

// Cases: f32_matCxR_[non_]const
// Note that f32 values may be not exactly representable in f16 and/or out of range.
const f32_mat_cases = [2, 3, 4].
flatMap((cols) =>
[2, 3, 4].flatMap((rows) =>
[true, false].map((nonConst) => ({
  [`f32_mat${cols}x${rows}_${nonConst ? 'non_const' : 'const'}`]: () => {
    return FP.f32.generateMatrixToMatrixCases(
      sparseMatrixF32Range(cols, rows),
      nonConst ? 'unfiltered' : 'finite',
      FP.f16.correctlyRoundedMatrix
    );
  }
}))
)
).
reduce((a, b) => ({ ...a, ...b }), {});

// Cases: f16_matCxR_[non_]const
const f16_mat_cases = [2, 3, 4].
flatMap((cols) =>
[2, 3, 4].flatMap((rows) =>
[true, false].map((nonConst) => ({
  [`f16_mat${cols}x${rows}_${nonConst ? 'non_const' : 'const'}`]: () => {
    // Input matrix is of f16 types, use f16.generateMatrixToMatrixCases.
    return FP.f16.generateMatrixToMatrixCases(
      sparseMatrixF16Range(cols, rows),
      nonConst ? 'unfiltered' : 'finite',
      FP.f16.correctlyRoundedMatrix
    );
  }
}))
)
).
reduce((a, b) => ({ ...a, ...b }), {});

export const d = makeCaseCache('unary/f16_conversion', {
  bool: () => {
    return [
    { input: bool(true), expected: f16(1.0) },
    { input: bool(false), expected: f16(0.0) }];

  },
  u32_non_const: () => {
    return [...fullU32Range(), 65504].map((u) => {
      return { input: u32(u), expected: FP.f16.correctlyRoundedInterval(u) };
    });
  },
  u32_const: () => {
    return [...fullU32Range(), 65504].
    filter((v) => f16FiniteRangeInterval.contains(v)).
    map((u) => {
      return { input: u32(u), expected: FP.f16.correctlyRoundedInterval(u) };
    });
  },
  i32_non_const: () => {
    return [...fullI32Range(), 65504, -65504].map((i) => {
      return { input: i32(i), expected: FP.f16.correctlyRoundedInterval(i) };
    });
  },
  i32_const: () => {
    return [...fullI32Range(), 65504, -65504].
    filter((v) => f16FiniteRangeInterval.contains(v)).
    map((i) => {
      return { input: i32(i), expected: FP.f16.correctlyRoundedInterval(i) };
    });
  },
  // Note that f32 values may be not exactly representable in f16 and/or out of range.
  f32_non_const: () => {
    return FP.f32.generateScalarToIntervalCases(
      [...scalarF32Range(), 65535.996, -65535.996],
      'unfiltered',
      FP.f16.correctlyRoundedInterval
    );
  },
  f32_const: () => {
    return FP.f32.generateScalarToIntervalCases(
      [...scalarF32Range(), 65535.996, -65535.996],
      'finite',
      FP.f16.correctlyRoundedInterval
    );
  },
  // All f16 values are exactly representable in f16.
  f16: () => {
    return scalarF16Range().map((f) => {
      return { input: f16(f), expected: FP.f16.correctlyRoundedInterval(f) };
    });
  },
  ...f32_mat_cases,
  ...f16_mat_cases
});