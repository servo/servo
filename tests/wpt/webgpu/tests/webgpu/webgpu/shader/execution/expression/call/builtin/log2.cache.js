/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { FP } from '../../../../../util/floating_point.js';import { biasedRange, linearRange } from '../../../../../util/math.js';import { makeCaseCache } from '../../case_cache.js';

// log2's accuracy is defined in three regions { [0, 0.5), [0.5, 2.0], (2.0, +∞] }
// Cases: [f32|f16]_[non_]const
const cases = ['f32', 'f16'].
flatMap((trait) =>
[true, false].map((nonConst) => ({
  [`${trait}_${nonConst ? 'non_const' : 'const'}`]: () => {
    return FP[trait].generateScalarToIntervalCases(
      [
      ...linearRange(FP[trait].constants().positive.min, 0.5, 20),
      ...linearRange(0.5, 2.0, 20),
      ...biasedRange(2.0, 2 ** 32, 1000),
      ...FP[trait].scalarRange()],

      nonConst ? 'unfiltered' : 'finite',
      FP[trait].log2Interval
    );
  }
}))
).
reduce((a, b) => ({ ...a, ...b }), {});

export const d = makeCaseCache('log2', cases);