/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { FP } from '../../../../../util/floating_point.js';import { makeCaseCache } from '../../case_cache.js';
// Cases: [f32|f16|abstract]
const cases = ['f32', 'f16', 'abstract'].
map((trait) => ({
  [`${trait}`]: () => {
    return FP[trait].generateScalarPairToIntervalCases(
      FP[trait].sparseScalarRange(),
      FP[trait].sparseScalarRange(),
      'unfiltered',
      FP[trait].minInterval
    );
  }
})).
reduce((a, b) => ({ ...a, ...b }), {});

export const d = makeCaseCache('min', cases);