/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { FP } from '../../../../../util/floating_point.js';import { fullU32Range } from '../../../../../util/math.js';import { makeCaseCache } from '../../case_cache.js';

export const d = makeCaseCache('unpack2x16float', {
  u32_const: () => {
    return FP.f32.generateU32ToIntervalCases(
      fullU32Range(),
      'finite',
      FP.f32.unpack2x16floatInterval
    );
  },
  u32_non_const: () => {
    return FP.f32.generateU32ToIntervalCases(
      fullU32Range(),
      'unfiltered',
      FP.f32.unpack2x16floatInterval
    );
  }
});