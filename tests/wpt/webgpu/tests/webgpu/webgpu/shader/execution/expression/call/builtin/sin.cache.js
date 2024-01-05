/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { FP } from '../../../../../util/floating_point.js';import { linearRange } from '../../../../../util/math.js';import { makeCaseCache } from '../../case_cache.js';

// Cases: [f32|f16]
const cases = ['f32', 'f16'].
map((trait) => ({
  [`${trait}`]: () => {
    return FP[trait].generateScalarToIntervalCases(
      [
      // Well-defined accuracy range
      ...linearRange(-Math.PI, Math.PI, 100),
      ...FP[trait].scalarRange()],

      'unfiltered',
      FP[trait].sinInterval
    );
  }
})).
reduce((a, b) => ({ ...a, ...b }), {});

export const d = makeCaseCache('sin', cases);