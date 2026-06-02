/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { FP } from '../../../../../util/floating_point.js';import { makeCaseCache } from '../../case_cache.js';
// See https://github.com/gpuweb/cts/issues/2766 for details
const kIssue2766Value = {
  abstract: 0x8000_0000_0000_0000,
  f32: 0x8000_0000,
  f16: 0x8000
};

// Cases: [f32|f16|abstract]
const cases = ['f32', 'f16', 'abstract'].
map((trait) => ({
  [`${trait}`]: () => {
    return FP[trait].generateScalarToIntervalCases(
      [kIssue2766Value[trait], ...FP[trait].scalarRange()],
      'unfiltered',
      FP[trait].roundInterval
    );
  }
})).
reduce((a, b) => ({ ...a, ...b }), {});

export const d = makeCaseCache('round', cases);