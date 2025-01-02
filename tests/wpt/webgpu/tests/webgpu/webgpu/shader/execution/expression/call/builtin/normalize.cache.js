/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { FP } from '../../../../../util/floating_point.js';import { makeCaseCache } from '../../case_cache.js';
// Cases: [f32|f16|abstract]_vecN_[non_]const
const cases = ['f32', 'f16', 'abstract'].
flatMap((trait) =>
[2, 3, 4].flatMap((dim) =>
[true, false].map((nonConst) => ({
  [`${trait}_vec${dim}_${nonConst ? 'non_const' : 'const'}`]: () => {
    if (trait === 'abstract' && nonConst) {
      return [];
    }
    return FP[trait].generateVectorToVectorCases(
      FP[trait].vectorRange(dim),
      nonConst ? 'unfiltered' : 'finite',
      // normalize has an inherited accuracy, so is only expected to be as accurate as f32
      FP[trait !== 'abstract' ? trait : 'f32'].normalizeInterval
    );
  }
}))
)
).
reduce((a, b) => ({ ...a, ...b }), {});

export const d = makeCaseCache('normalize', cases);