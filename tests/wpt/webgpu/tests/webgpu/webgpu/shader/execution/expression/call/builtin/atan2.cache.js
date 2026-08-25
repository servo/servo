/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { FP } from '../../../../../util/floating_point.js';import { linearRange } from '../../../../../util/math.js';import { makeCaseCache } from '../../case_cache.js';

// Cases: [f32|f16|abstract]_[non_]const
const cases = ['f32', 'f16', 'abstract'].
flatMap((trait) =>
[true, false].map((nonConst) => ({
  [`${trait}_${nonConst ? 'non_const' : 'const'}`]: () => {
    if (trait === 'abstract' && nonConst) {
      return [];
    }
    // Using sparse range since there are N^2 cases being generated, and also including extra values
    // around 0, where there is a discontinuity that implementations may behave badly at.
    const numeric_range = [
    ...FP[trait].sparseScalarRange(),
    ...linearRange(
      FP[trait].constants().negative.max,
      FP[trait].constants().positive.min,
      10
    )];

    return FP[trait].generateScalarPairToIntervalCases(
      numeric_range,
      numeric_range,
      nonConst ? 'unfiltered' : 'finite',
      // atan2 has an ulp accuracy, so is only expected to be as accurate as f32
      FP[trait !== 'abstract' ? trait : 'f32'].atan2Interval
    );
  }
}))
).
reduce((a, b) => ({ ...a, ...b }), {});

export const d = makeCaseCache('atan2', cases);