/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { FP } from '../../../../../util/floating_point.js';import { selectNCases } from '../../case.js';import { makeCaseCache } from '../../case_cache.js';

// Cases: [f32|f16|abstract]_[non_]const
// abstract_non_const is empty and unused
const scalar_cases = ['f32', 'f16', 'abstract'].
flatMap((trait) =>
[true, false].map((nonConst) => ({
  [`${trait}_${nonConst ? 'non_const' : 'const'}`]: () => {
    if (trait === 'abstract' && nonConst) {
      return [];
    }
    const cases = FP[trait].generateScalarTripleToIntervalCases(
      FP[trait].sparseScalarRange(),
      FP[trait].sparseScalarRange(),
      FP[trait].sparseScalarRange(),
      nonConst ? 'unfiltered' : 'finite',
      // mix has an inherited accuracy, so abstract is only expected to be as accurate as f32
      ...FP[trait !== 'abstract' ? trait : 'f32'].mixIntervals
    );
    return selectNCases('mix_scalar', trait === 'abstract' ? 50 : cases.length, cases);
  }
}))
).
reduce((a, b) => ({ ...a, ...b }), {});

// Cases: [f32|f16]_vecN_scalar_[non_]const
// abstract_vecN_non_const is empty and unused
const vec_scalar_cases = ['f32', 'f16', 'abstract'].
flatMap((trait) =>
[2, 3, 4].flatMap((dim) =>
[true, false].map((nonConst) => ({
  [`${trait}_vec${dim}_scalar_${nonConst ? 'non_const' : 'const'}`]: () => {
    if (trait === 'abstract' && nonConst) {
      return [];
    }
    const cases = FP[trait].generateVectorPairScalarToVectorComponentWiseCase(
      FP[trait].sparseVectorRange(dim),
      FP[trait].sparseVectorRange(dim),
      FP[trait].sparseScalarRange(),
      nonConst ? 'unfiltered' : 'finite',
      // mix has an inherited accuracy, so abstract is only expected to be as accurate as f32
      ...FP[trait !== 'abstract' ? trait : 'f32'].mixIntervals
    );
    return selectNCases('mix_vector', trait === 'abstract' ? 50 : cases.length, cases);
  }
}))
)
).
reduce((a, b) => ({ ...a, ...b }), {});

export const d = makeCaseCache('mix', {
  ...scalar_cases,
  ...vec_scalar_cases
});