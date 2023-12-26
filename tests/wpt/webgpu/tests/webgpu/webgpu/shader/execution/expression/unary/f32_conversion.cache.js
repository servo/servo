/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { bool, f16, f32, i32, u32 } from '../../../../util/conversion.js';import { FP } from '../../../../util/floating_point.js';import {
  fullI32Range,
  fullU32Range,
  scalarF16Range,
  scalarF32Range,
  sparseMatrixF16Range,
  sparseMatrixF32Range } from
'../../../../util/math.js';
import { makeCaseCache } from '../case_cache.js';

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
  ...f32_mat_cases,
  ...f16_mat_cases
});