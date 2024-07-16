/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { FP } from '../../../../../util/floating_point.js';import { makeCaseCache } from '../../case_cache.js';
// Cases: [f32|f16|abstract]
const cases = ['f32', 'f16', 'abstract'].
map((trait) => ({
  [`${trait}`]: () => {
    return FP[trait].generateScalarToIntervalCases(
      FP[trait].scalarRange(),
      'unfiltered',
      // tanh has an inherited accuracy, so is only expected to be as accurate as f32
      FP[trait !== 'abstract' ? trait : 'f32'].tanhInterval
    );
  }
})).
reduce((a, b) => ({ ...a, ...b }), {});

export const d = makeCaseCache('tanh', cases);