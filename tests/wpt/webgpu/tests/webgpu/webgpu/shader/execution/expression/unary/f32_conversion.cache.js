/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { abstractInt, bool, f16, f32, i32, u32 } from '../../../../util/conversion.js';import { FP, FPInterval } from '../../../../util/floating_point.js';import {
  fullI32Range,
  fullI64Range,
  fullU32Range,
  scalarF16Range,
  scalarF32Range,
  sparseMatrixF16Range,
  sparseMatrixF32Range } from
'../../../../util/math.js';
import { makeCaseCache } from '../case_cache.js';

const f32FiniteRangeInterval = new FPInterval(
  'f32',
  FP.f32.constants().negative.min,
  FP.f32.constants().positive.max
);

// Cases: f32_matCxR_[non_]const
const f32_mat_cases = [2, 3, 4].
flatMap((cols) =>
[2, 3, 4].flatMap((rows) =>
[true, false].map((nonConst) => ({
  [`f32_mat${cols}x${rows}_${nonConst ? 'non_const' : 'const'}`]: () => {
    return FP.f32.generateMatrixToMatrixCases(
      sparseMatrixF32Range(cols, rows),
      nonConst ? 'unfiltered' : 'finite',
      FP.f32.correctlyRoundedMatrix
    );
  }
}))
)
).
reduce((a, b) => ({ ...a, ...b }), {});

// Cases: f16_matCxR_[non_]const
// Note that all f16 values are exactly representable in f32.
const f16_mat_cases = [2, 3, 4].
flatMap((cols) =>
[2, 3, 4].flatMap((rows) =>
[true, false].map((nonConst) => ({
  [`f16_mat${cols}x${rows}_${nonConst ? 'non_const' : 'const'}`]: () => {
    // Input matrix is of f16 types, use f16.generateMatrixToMatrixCases.
    return FP.f16.generateMatrixToMatrixCases(
      sparseMatrixF16Range(cols, rows),
      nonConst ? 'unfiltered' : 'finite',
      FP.f32.correctlyRoundedMatrix
    );
  }
}))
)
).
reduce((a, b) => ({ ...a, ...b }), {});

// Cases: abstract_float_matCxR
// Note that abstract float values may be not exactly representable in f32
// and/or out of range.
const abstract_float_mat_cases = [2, 3, 4].
flatMap((cols) =>
[2, 3, 4].map((rows) => ({
  [`abstract_float_mat${cols}x${rows}`]: () => {
    return FP.abstract.generateMatrixToMatrixCases(
      FP.abstract.sparseMatrixRange(cols, rows),
      'finite',
      FP.f32.correctlyRoundedMatrix
    );
  }
}))
).
reduce((a, b) => ({ ...a, ...b }), {});

export const d = makeCaseCache('unary/f32_conversion', {
  bool: () => {
    return [
    { input: bool(true), expected: f32(1.0) },
    { input: bool(false), expected: f32(0.0) }];

  },
  u32: () => {
    return fullU32Range().map((u) => {
      return { input: u32(u), expected: FP.f32.correctlyRoundedInterval(u) };
    });
  },
  i32: () => {
    return fullI32Range().map((i) => {
      return { input: i32(i), expected: FP.f32.correctlyRoundedInterval(i) };
    });
  },
  abstract_int: () => {
    return [...fullI64Range()].
    filter((v) => f32FiniteRangeInterval.contains(Number(v))).
    map((i) => {
      return { input: abstractInt(i), expected: FP.f32.correctlyRoundedInterval(Number(i)) };
    });
  },
  f32: () => {
    return scalarF32Range().map((f) => {
      return { input: f32(f), expected: FP.f32.correctlyRoundedInterval(f) };
    });
  },
  // All f16 values are exactly representable in f32.
  f16: () => {
    return scalarF16Range().map((f) => {
      return { input: f16(f), expected: FP.f32.correctlyRoundedInterval(f) };
    });
  },
  // Note that abstract float values may be not exactly representable in f32.
  abstract_float: () => {
    return FP.abstract.generateScalarToIntervalCases(
      [...FP.abstract.scalarRange()],
      'finite',
      FP.f32.correctlyRoundedInterval
    );
  },
  ...f32_mat_cases,
  ...f16_mat_cases,
  ...abstract_float_mat_cases
});