/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { toVector } from '../../../../../util/conversion.js';import { FP } from '../../../../../util/floating_point.js';
import { makeCaseCache } from '../../case_cache.js';

/** @returns a fract Case for a scalar vector input */
function makeScalarCaseFract(kind, n) {
  const fp = FP[kind];
  n = fp.quantize(n);
  const result = fp.modfInterval(n).fract;

  return { input: fp.scalarBuilder(n), expected: result };
}

/** @returns a whole Case for a scalar vector input */
function makeScalarCaseWhole(kind, n) {
  const fp = FP[kind];
  n = fp.quantize(n);
  const result = fp.modfInterval(n).whole;

  return { input: fp.scalarBuilder(n), expected: result };
}

/** @returns a fract Case for a given vector input */
function makeVectorCaseFract(kind, v) {
  const fp = FP[kind];
  v = v.map(fp.quantize);
  const fs = v.map((e) => {
    return fp.modfInterval(e).fract;
  });

  return { input: toVector(v, fp.scalarBuilder), expected: fs };
}

/** @returns a whole Case for a given vector input */
function makeVectorCaseWhole(kind, v) {
  const fp = FP[kind];
  v = v.map(fp.quantize);
  const ws = v.map((e) => {
    return fp.modfInterval(e).whole;
  });

  return { input: toVector(v, fp.scalarBuilder), expected: ws };
}

// Cases: [f32|f16|abstract]_[fract|whole]
const scalar_cases = ['f32', 'f16', 'abstract'].
flatMap((kind) =>
['whole', 'fract'].map((portion) => ({
  [`${kind}_${portion}`]: () => {
    const makeCase = portion === 'whole' ? makeScalarCaseWhole : makeScalarCaseFract;
    return FP[kind].scalarRange().map(makeCase.bind(null, kind));
  }
}))
).
reduce((a, b) => ({ ...a, ...b }), {});

// Cases: [f32|f16|abstract]_vecN_[fract|whole]
const vec_cases = ['f32', 'f16', 'abstract'].
flatMap((kind) =>
[2, 3, 4].flatMap((n) =>
['whole', 'fract'].map((portion) => ({
  [`${kind}_vec${n}_${portion}`]: () => {
    const makeCase = portion === 'whole' ? makeVectorCaseWhole : makeVectorCaseFract;
    return FP[kind].vectorRange(n).map(makeCase.bind(null, kind));
  }
}))
)
).
reduce((a, b) => ({ ...a, ...b }), {});

export const d = makeCaseCache('modf', {
  ...scalar_cases,
  ...vec_cases
});