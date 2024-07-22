/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { FP } from '../../../../../util/floating_point.js';import { biasedRange, linearRange } from '../../../../../util/math.js';import { makeCaseCache } from '../../case_cache.js';

// log's accuracy is defined in three regions { [0, 0.5), [0.5, 2.0], (2.0, +∞] }
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
      ...linearRange(FP[trait].constants().positive.min, 0.5, 20),
      ...linearRange(0.5, 2.0, 20),
      ...biasedRange(2.0, 2 ** 32, 1000),
      ...FP[trait].scalarRange()],

      nonConst ? 'unfiltered' : 'finite',
      // log has an absolute or ulp accuracy, so is only expected to be as accurate as f32
      FP[trait !== 'abstract' ? trait : 'f32'].logInterval
    );
  }
}))
).
reduce((a, b) => ({ ...a, ...b }), {});

export const d = makeCaseCache('log', cases);