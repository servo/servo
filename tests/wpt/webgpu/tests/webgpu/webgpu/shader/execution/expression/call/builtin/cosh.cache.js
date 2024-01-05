/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { FP } from '../../../../../util/floating_point.js';import { makeCaseCache } from '../../case_cache.js';
// Cases: [f32|f16]
const cases = ['f32', 'f16'].
flatMap((trait) =>
[true, false].map((nonConst) => ({
  [`${trait}_${nonConst ? 'non_const' : 'const'}`]: () => {
    return FP[trait].generateScalarToIntervalCases(
      FP[trait].scalarRange(),
      nonConst ? 'unfiltered' : 'finite',
      FP[trait].coshInterval
    );
  }
}))
).
reduce((a, b) => ({ ...a, ...b }), {});

export const d = makeCaseCache('cosh', cases);