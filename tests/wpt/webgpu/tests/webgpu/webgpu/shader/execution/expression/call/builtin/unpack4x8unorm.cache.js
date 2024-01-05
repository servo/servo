/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { FP } from '../../../../../util/floating_point.js';import { fullU32Range } from '../../../../../util/math.js';import { makeCaseCache } from '../../case_cache.js';

export const d = makeCaseCache('unpack4x8unorm', {
  u32_const: () => {
    return FP.f32.generateU32ToIntervalCases(
      fullU32Range(),
      'finite',
      FP.f32.unpack4x8unormInterval
    );
  },
  u32_non_const: () => {
    return FP.f32.generateU32ToIntervalCases(
      fullU32Range(),
      'unfiltered',
      FP.f32.unpack4x8unormInterval
    );
  }
});