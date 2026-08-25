/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { abstractInt } from '../../../../../util/conversion.js';import { FP } from '../../../../../util/floating_point.js';import { absBigInt, fullI64Range } from '../../../../../util/math.js';
import { makeCaseCache } from '../../case_cache.js';

// Cases: [f32|f16|abstract_float|abstract_int]
const cases = {
  ...['f32', 'f16', 'abstract'].
  map((trait) => ({
    [`${trait === 'abstract' ? 'abstract_float' : trait}`]: () => {
      return FP[trait].generateScalarToIntervalCases(
        FP[trait].scalarRange(),
        'unfiltered',
        FP[trait].absInterval
      );
    }
  })).
  reduce((a, b) => ({ ...a, ...b }), {}),
  abstract_int: () => {
    return fullI64Range().map((e) => {
      return { input: abstractInt(e), expected: abstractInt(absBigInt(e)) };
    });
  }
};

export const d = makeCaseCache('abs', cases);