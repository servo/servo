/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { FP } from '../../../../util/floating_point.js';import { sparseScalarF64Range, sparseVectorF64Range } from '../../../../util/math.js';import { makeCaseCache } from '../case_cache.js';

const subtractionVectorScalarInterval = (v, s) => {
  return FP.abstract.toVector(v.map((e) => FP.abstract.subtractionInterval(e, s)));
};

const subtractionScalarVectorInterval = (s, v) => {
  return FP.abstract.toVector(v.map((e) => FP.abstract.subtractionInterval(s, e)));
};

const scalar_cases = {
  ['scalar']: () => {
    return FP.abstract.generateScalarPairToIntervalCases(
      sparseScalarF64Range(),
      sparseScalarF64Range(),
      'finite',
      FP.abstract.subtractionInterval
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
      subtractionVectorScalarInterval
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
      subtractionScalarVectorInterval
    );
  }
})).
reduce((a, b) => ({ ...a, ...b }), {});

export const d = makeCaseCache('binary/af_subtraction', {
  ...scalar_cases,
  ...vector_scalar_cases,
  ...scalar_vector_cases
});