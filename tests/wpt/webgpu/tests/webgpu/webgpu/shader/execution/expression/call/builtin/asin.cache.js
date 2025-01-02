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
    return FP[trait].generateScalarToIntervalCases(
      [...linearRange(-1, 1, 100), ...FP[trait].scalarRange()], // asin is defined on [-1, 1]
      nonConst ? 'unfiltered' : 'finite',
      // asin has an ulp or inherited accuracy, so is only expected to be as accurate as f32
      FP[trait !== 'abstract' ? trait : 'f32'].asinInterval
    );
  }
}))
).
reduce((a, b) => ({ ...a, ...b }), {});

export const d = makeCaseCache('asin', cases);