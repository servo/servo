/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { kValue } from '../../../../../util/constants.js';import { FP } from '../../../../../util/floating_point.js';import { scalarF16Range, scalarF32Range } from '../../../../../util/math.js';
import { makeCaseCache } from '../../case_cache.js';

export const d = makeCaseCache('quantizeToF16', {
  f32_const: () => {
    return FP.f32.generateScalarToIntervalCases(
      [
      kValue.f16.negative.min,
      kValue.f16.negative.max,
      kValue.f16.negative.subnormal.min,
      kValue.f16.negative.subnormal.max,
      kValue.f16.positive.subnormal.min,
      kValue.f16.positive.subnormal.max,
      kValue.f16.positive.min,
      kValue.f16.positive.max,
      ...scalarF16Range()],

      'finite',
      FP.f32.quantizeToF16Interval
    );
  },
  f32_non_const: () => {
    return FP.f32.generateScalarToIntervalCases(
      [
      kValue.f16.negative.min,
      kValue.f16.negative.max,
      kValue.f16.negative.subnormal.min,
      kValue.f16.negative.subnormal.max,
      kValue.f16.positive.subnormal.min,
      kValue.f16.positive.subnormal.max,
      kValue.f16.positive.min,
      kValue.f16.positive.max,
      ...scalarF32Range()],

      'unfiltered',
      FP.f32.quantizeToF16Interval
    );
  }
});