/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { FP } from '../../../../../util/floating_point.js';import { biasedRange } from '../../../../../util/math.js';import { makeCaseCache } from '../../case_cache.js';

// Cases: [f32|f16|abstract]_[non_]const
const cases = ['f32', 'f16', 'abstract'].
flatMap((trait) =>
[true, false].map((nonConst) => ({
  [`${trait}_${nonConst ? 'non_const' : 'const'}`]: () => {
    if (trait === 'abstract' && nonConst) {
      return [];
    }
    return FP[trait].generateScalarToIntervalCases(
      [
      // discontinuity at x = -1
      ...biasedRange(FP[trait].constants().negative.less_than_one, -0.9, 20),
      -1,
      // discontinuity at x = 1
      ...biasedRange(FP[trait].constants().positive.less_than_one, 0.9, 20),
      1,
      ...FP[trait].scalarRange()],

      nonConst ? 'unfiltered' : 'finite',
      // atanh has an inherited accuracy, so is only expected to be as accurate as f32
      FP[trait !== 'abstract' ? trait : 'f32'].atanhInterval
    );
  }
}))
).
reduce((a, b) => ({ ...a, ...b }), {});

export const d = makeCaseCache('atanh', cases);