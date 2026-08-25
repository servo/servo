/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { FP } from '../../../../util/floating_point.js';import { sparseScalarF64Range, sparseVectorF64Range } from '../../../../util/math.js';import { makeCaseCache } from '../case_cache.js';

const remainderVectorScalarInterval = (v, s) => {
  // remainder has an inherited accuracy, so abstract is only expected to be as accurate as f32
  return FP.abstract.toVector(v.map((e) => FP.f32.remainderInterval(e, s)));
};

const remainderScalarVectorInterval = (s, v) => {
  // remainder has an inherited accuracy, so abstract is only expected to be as accurate as f32
  return FP.abstract.toVector(v.map((e) => FP.f32.remainderInterval(s, e)));
};

const scalar_cases = {
  ['scalar']: () => {
    return FP.abstract.generateScalarPairToIntervalCases(
      sparseScalarF64Range(),
      sparseScalarF64Range(),
      'finite',
      // remainder has an inherited accuracy, so abstract is only expected to be as accurate as f32
      FP.f32.remainderInterval
    );
  }
};

const vector_scalar_cases = [2, 3, 4].
map((dim) => ({
  [`vec${dim}_scalar`]: () => {
    return FP.abstract.generateVectorScalarToVectorCases(
      sparseVectorF64Range(dim),
      sparseScalarF64Range(),
      'finite',
      remainderVectorScalarInterval
    );
  }
})).
reduce((a, b) => ({ ...a, ...b }), {});

const scalar_vector_cases = [2, 3, 4].
map((dim) => ({
  [`scalar_vec${dim}`]: () => {
    return FP.abstract.generateScalarVectorToVectorCases(
      sparseScalarF64Range(),
      sparseVectorF64Range(dim),
      'finite',
      remainderScalarVectorInterval
    );
  }
})).
reduce((a, b) => ({ ...a, ...b }), {});

export const d = makeCaseCache('binary/af_remainder', {
  ...scalar_cases,
  ...vector_scalar_cases,
  ...scalar_vector_cases
});