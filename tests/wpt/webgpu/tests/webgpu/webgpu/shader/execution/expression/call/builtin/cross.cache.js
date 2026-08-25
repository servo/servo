/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { FP } from '../../../../../util/floating_point.js';import { makeCaseCache } from '../../case_cache.js';
// Cases: [f32|f16|abstract]_[non_]const
// abstract_non_const is empty and not used
const cases = ['f32', 'f16', 'abstract'].
flatMap((trait) =>
[true, false].map((nonConst) => ({
  [`${trait}_${nonConst ? 'non_const' : 'const'}`]: () => {
    if (trait === 'abstract' && nonConst) {
      return [];
    }
    return FP[trait].generateVectorPairToVectorCases(
      FP[trait].vectorRange(3),
      FP[trait].vectorRange(3),
      nonConst ? 'unfiltered' : 'finite',
      // cross has an inherited accuracy, so abstract is only expected to be as accurate as f32
      FP[trait !== 'abstract' ? trait : 'f32'].crossInterval
    );
  }
}))
).
reduce((a, b) => ({ ...a, ...b }), {});

export const d = makeCaseCache('cross', cases);