/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { FP } from '../../../../../util/floating_point.js';import { makeCaseCache } from '../../case_cache.js';
// Cases: [f32|f16]_[non_]const
const scalar_cases = ['f32', 'f16'].
flatMap((trait) =>
[true, false].map((nonConst) => ({
  [`${trait}_${nonConst ? 'non_const' : 'const'}`]: () => {
    return FP[trait].generateScalarPairToIntervalCases(
      FP[trait].scalarRange(),
      FP[trait].scalarRange(),
      nonConst ? 'unfiltered' : 'finite',
      FP[trait].distanceInterval
    );
  }
}))
).
reduce((a, b) => ({ ...a, ...b }), {});

// Cases: [f32|f16]_vecN_[non_]const
const vec_cases = ['f32', 'f16'].
flatMap((trait) =>
[2, 3, 4].flatMap((dim) =>
[true, false].map((nonConst) => ({
  [`${trait}_vec${dim}_${nonConst ? 'non_const' : 'const'}`]: () => {
    return FP[trait].generateVectorPairToIntervalCases(
      FP[trait].sparseVectorRange(dim),
      FP[trait].sparseVectorRange(dim),
      nonConst ? 'unfiltered' : 'finite',
      FP[trait].distanceInterval
    );
  }
}))
)
).
reduce((a, b) => ({ ...a, ...b }), {});

export const d = makeCaseCache('distance', {
  ...scalar_cases,
  ...vec_cases
});