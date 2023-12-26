/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { FP } from '../../../../../util/floating_point.js';import { makeCaseCache } from '../../case_cache.js';
// Cases: [f32|f16]_vecN_[non_]const
const cases = ['f32', 'f16'].
flatMap((trait) =>
[2, 3, 4].flatMap((N) =>
[true, false].map((nonConst) => ({
  [`${trait}_vec${N}_${nonConst ? 'non_const' : 'const'}`]: () => {
    // vec3 and vec4 require calculating all possible permutations, so their runtime is much
    // longer per test, so only using sparse vectors for them.
    return FP[trait].generateVectorPairToIntervalCases(
      N === 2 ? FP[trait].vectorRange(2) : FP[trait].sparseVectorRange(N),
      N === 2 ? FP[trait].vectorRange(2) : FP[trait].sparseVectorRange(N),
      nonConst ? 'unfiltered' : 'finite',
      FP[trait].dotInterval
    );
  }
}))
)
).
reduce((a, b) => ({ ...a, ...b }), {});

export const d = makeCaseCache('dot', cases);