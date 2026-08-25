/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { FP } from '../../../../../util/floating_point.js';import { linearRange } from '../../../../../util/math.js';import { makeCaseCache } from '../../case_cache.js';

// Cases: [f32|f16|abstract]
const cases = ['f32', 'f16', 'abstract'].
map((trait) => ({
  [`${trait}`]: () => {
    return FP[trait].generateScalarToIntervalCases(
      [
      // Well-defined accuracy range
      ...linearRange(-Math.PI, Math.PI, 100),
      ...FP[trait].scalarRange()],

      trait === 'abstract' ? 'finite' : 'unfiltered',
      // cos has an absolute accuracy, so is only expected to be as accurate as f32
      FP[trait !== 'abstract' ? trait : 'f32'].cosInterval
    );
  }
})).
reduce((a, b) => ({ ...a, ...b }), {});

export const d = makeCaseCache('cos', cases);