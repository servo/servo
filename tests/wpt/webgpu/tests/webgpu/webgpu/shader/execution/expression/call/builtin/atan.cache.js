/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { FP } from '../../../../../util/floating_point.js';import { makeCaseCache } from '../../case_cache.js';
const known_values = [-Math.sqrt(3), -1, -1 / Math.sqrt(3), 0, 1, 1 / Math.sqrt(3), Math.sqrt(3)];

// Cases: [f32|f16]_[non_]const
const cases = ['f32', 'f16'].
flatMap((trait) =>
[true, false].map((nonConst) => ({
  [`${trait}_${nonConst ? 'non_const' : 'const'}`]: () => {
    return FP[trait].generateScalarToIntervalCases(
      [...known_values, ...FP[trait].scalarRange()],
      nonConst ? 'unfiltered' : 'finite',
      FP[trait].atanInterval
    );
  }
}))
).
reduce((a, b) => ({ ...a, ...b }), {});

export const d = makeCaseCache('atan', cases);