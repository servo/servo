/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { FP } from '../../../../../util/floating_point.js';import { sparseScalarF32Range } from '../../../../../util/math.js';import { makeCaseCache } from '../../case_cache.js';

export const d = makeCaseCache('derivatives', {
  scalar: () => {
    return FP.f32.generateScalarPairToIntervalCases(
      sparseScalarF32Range(),
      sparseScalarF32Range(),
      'unfiltered',
      FP.f32.subtractionInterval
    );
  }
});