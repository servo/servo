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
      [...biasedRange(1, 2, 100), ...FP[trait].scalarRange()], // x near 1 can be problematic to implement
      nonConst ? 'unfiltered' : 'finite',
      // acosh has an inherited accuracy, so is only expected to be as accurate as f32
      ...FP[trait !== 'abstract' ? trait : 'f32'].acoshIntervals
    );
  }
}))
).
reduce((a, b) => ({ ...a, ...b }), {});

export const d = makeCaseCache('acosh', cases);