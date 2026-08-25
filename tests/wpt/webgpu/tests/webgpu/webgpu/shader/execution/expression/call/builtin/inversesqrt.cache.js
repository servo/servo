/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { kValue } from '../../../../../util/constants.js';import { FP } from '../../../../../util/floating_point.js';import { biasedRange, linearRange } from '../../../../../util/math.js';
import { makeCaseCache } from '../../case_cache.js';

export const d = makeCaseCache('inverseSqrt', {
  f32: () => {
    return FP.f32.generateScalarToIntervalCases(
      [
      // 0 < x <= 1 linearly spread
      ...linearRange(kValue.f32.positive.min, 1, 100),
      // 1 <= x < 2^32, biased towards 1
      ...biasedRange(1, 2 ** 32, 1000)],

      'unfiltered',
      FP.f32.inverseSqrtInterval
    );
  },
  f16: () => {
    return FP.f16.generateScalarToIntervalCases(
      [
      // 0 < x <= 1 linearly spread
      ...linearRange(kValue.f16.positive.min, 1, 100),
      // 1 <= x < 2^15, biased towards 1
      ...biasedRange(1, 2 ** 15, 1000)],

      'unfiltered',
      FP.f16.inverseSqrtInterval
    );
  },
  abstract: () => {
    return FP.abstract.generateScalarToIntervalCases(
      [
      // 0 < x <= 1 linearly spread
      ...linearRange(kValue.f64.positive.min, 1, 100),
      // 1 <= x < 2^64, biased towards 1, only using 100 steps, because af tests are more expensive per case
      ...biasedRange(1, 2 ** 64, 100)],

      'finite',
      // inverseSqrt has an ulp accuracy, so is only expected to be as accurate as f32
      FP.f32.inverseSqrtInterval
    );
  }
});