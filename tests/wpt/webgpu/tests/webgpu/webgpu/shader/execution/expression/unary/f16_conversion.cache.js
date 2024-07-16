/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { abstractInt, bool, f16, i32, u32 } from '../../../../util/conversion.js';import { FP, FPInterval } from '../../../../util/floating_point.js';import { fullI32Range, fullI64Range, fullU32Range } from '../../../../util/math.js';
import { makeCaseCache } from '../case_cache.js';

const f16FiniteRangeInterval = new FPInterval(
  'f16',
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
      FP.f32.sparseMatrixRange(cols, rows),
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
      FP.f16.sparseMatrixRange(cols, rows),
      nonConst ? 'unfiltered' : 'finite',
      FP.f16.correctlyRoundedMatrix
    );
  }
}))
)
).
reduce((a, b) => ({ ...a, ...b }), {});

// Cases: abstract_float_matCxR
// Note that abstract float values may be not exactly representable in f16
// and/or out of range.
const abstract_float_mat_cases = [2, 3, 4].
flatMap((cols) =>
[2, 3, 4].map((rows) => ({
  [`abstract_float_mat${cols}x${rows}`]: () => {
    return FP.abstract.generateMatrixToMatrixCases(
      FP.abstract.sparseMatrixRange(cols, rows),
      'finite',
      FP.f16.correctlyRoundedMatrix
    );
  }
}))
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
  abstract_int: () => {
    return [...fullI64Range(), 65504n, -65504n].
    filter((v) => f16FiniteRangeInterval.contains(Number(v))).
    map((i) => {
      return { input: abstractInt(i), expected: FP.f16.correctlyRoundedInterval(Number(i)) };
    });
  },
  // Note that f32 values may be not exactly representable in f16 and/or out of range.
  f32_non_const: () => {
    return FP.f32.generateScalarToIntervalCases(
      [...FP.f32.scalarRange(), 65535.996, -65535.996],
      'unfiltered',
      FP.f16.correctlyRoundedInterval
    );
  },
  f32_const: () => {
    return FP.f32.generateScalarToIntervalCases(
      [...FP.f32.scalarRange(), 65535.996, -65535.996],
      'finite',
      FP.f16.correctlyRoundedInterval
    );
  },
  // Note that abstract float values may be not exactly representable in f16.
  abstract_float: () => {
    return FP.abstract.generateScalarToIntervalCases(
      [...FP.abstract.scalarRange(), 65535.996, -65535.996],
      'finite',
      FP.f16.correctlyRoundedInterval
    );
  },
  // All f16 values are exactly representable in f16.
  f16: () => {
    return FP.f16.scalarRange().map((f) => {
      return { input: f16(f), expected: FP.f16.correctlyRoundedInterval(f) };
    });
  },
  ...f32_mat_cases,
  ...f16_mat_cases,
  ...abstract_float_mat_cases
});