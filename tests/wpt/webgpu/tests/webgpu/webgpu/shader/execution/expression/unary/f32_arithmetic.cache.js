/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { FP } from '../../../../util/floating_point.js';import { scalarF32Range } from '../../../../util/math.js';import { makeCaseCache } from '../case_cache.js';

export const d = makeCaseCache('unary/f32_arithmetic', {
  negation: () => {
    return FP.f32.generateScalarToIntervalCases(
      scalarF32Range({ neg_norm: 250, neg_sub: 20, pos_sub: 20, pos_norm: 250 }),
      'unfiltered',
      FP.f32.negationInterval
    );
  }
});