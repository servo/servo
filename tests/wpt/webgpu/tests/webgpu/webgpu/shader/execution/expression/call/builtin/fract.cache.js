/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { assert } from '../../../../../../common/util/util.js';import { FP } from '../../../../../util/floating_point.js';import { kFractTable } from '../../binary/af_data.js';
import { makeCaseCache } from '../../case_cache.js';

const kCommonValues = [
0.5, // 0.5 -> 0.5
0.9, // ~0.9 -> ~0.9
1, // 1 -> 0
2, // 2 -> 0
1.11, // ~1.11 -> ~0.11
-0.1, // ~-0.1 -> ~0.9
-0.5, // -0.5 -> 0.5
-0.9, // ~-0.9 -> ~0.1
-1, // -1 -> 0
-2, // -2 -> 0
-1.11 // ~-1.11 -> ~0.89
];

const kTraitSpecificValues = {
  f32: [
  10.0001, // ~10.0001 -> ~0.0001
  -10.0001, // -10.0001 -> ~0.9999
  0x8000_0000 // https://github.com/gpuweb/cts/issues/2766
  ],
  f16: [
  10.0078125, // 10.0078125 -> 0.0078125
  -10.0078125, // -10.0078125 -> 0.9921875
  658.5, // 658.5 -> 0.5
  0x8000 // https://github.com/gpuweb/cts/issues/2766
  ]
};

// Cases: [f32|f16|abstract]
const concrete_cases = ['f32', 'f16'].
map((trait) => ({
  [`${trait}`]: () => {
    return FP[trait].generateScalarToIntervalCases(
      [...kCommonValues, ...kTraitSpecificValues[trait], ...FP[trait].scalarRange()],
      'unfiltered',
      FP[trait].fractInterval
    );
  }
})).
reduce((a, b) => ({ ...a, ...b }), {});

// Cases: [abstract]
const abstract_cases = () => {
  return FP.abstract.generateScalarToIntervalCases(
    [...kFractTable.keys()],
    'finite',
    (x) => {
      assert(kFractTable.has(x));
      return kFractTable.get(x);
    }
  );
};

export const d = makeCaseCache('fract', {
  abstract: abstract_cases,
  ...concrete_cases
});