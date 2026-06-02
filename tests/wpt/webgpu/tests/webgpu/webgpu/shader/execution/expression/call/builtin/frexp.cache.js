/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { skipUndefined } from '../../../../../util/compare.js';import {

  i32,
  toVector,
  abstractInt } from
'../../../../../util/conversion.js';
import { FP } from '../../../../../util/floating_point.js';
import { frexp } from '../../../../../util/math.js';

import { makeCaseCache } from '../../case_cache.js';

/* @returns a fract Case for a given scalar or vector input */
function makeCaseFract(v, trait) {
  const fp = FP[trait];
  let toInput;
  let toOutput;
  if (v instanceof Array) {
    // Input is vector
    toInput = (n) => toVector(n, fp.scalarBuilder);
    toOutput = (n) => toVector(n, fp.scalarBuilder);
  } else {
    // Input is scalar, also wrap it in an array.
    v = [v];
    toInput = (n) => fp.scalarBuilder(n[0]);
    toOutput = (n) => fp.scalarBuilder(n[0]);
  }

  v = v.map(fp.quantize);
  if (v.some((e) => e !== 0 && fp.isSubnormal(e))) {
    return { input: toInput(v), expected: skipUndefined(undefined) };
  }

  const fs = v.map((e) => {
    return frexp(e, trait !== 'abstract' ? trait : 'f64').fract;
  });

  return { input: toInput(v), expected: toOutput(fs) };
}

/* @returns an exp Case for a given scalar or vector input */
function makeCaseExp(v, trait) {
  const fp = FP[trait];
  let toInput;
  let toOutput;
  if (v instanceof Array) {
    // Input is vector
    toInput = (n) => toVector(n, fp.scalarBuilder);
    toOutput = (n) =>
    toVector(n, trait !== 'abstract' ? i32 : (n) => abstractInt(BigInt(n)));
  } else {
    // Input is scalar, also wrap it in an array.
    v = [v];
    toInput = (n) => fp.scalarBuilder(n[0]);
    toOutput = (n) =>
    trait !== 'abstract' ? i32(n[0]) : abstractInt(BigInt(n[0]));
  }

  v = v.map(fp.quantize);
  if (v.some((e) => e !== 0 && fp.isSubnormal(e))) {
    return { input: toInput(v), expected: skipUndefined(undefined) };
  }

  const fs = v.map((e) => {
    return frexp(e, trait !== 'abstract' ? trait : 'f64').exp;
  });

  return { input: toInput(v), expected: toOutput(fs) };
}

// Cases: [f32|f16]_vecN_[exp|whole]
const vec_cases = ['f32', 'f16', 'abstract'].
flatMap((trait) =>
[2, 3, 4].flatMap((dim) =>
['exp', 'fract'].map((portion) => ({
  [`${trait}_vec${dim}_${portion}`]: () => {
    return FP[trait].
    vectorRange(dim).
    map((v) => portion === 'exp' ? makeCaseExp(v, trait) : makeCaseFract(v, trait));
  }
}))
)
).
reduce((a, b) => ({ ...a, ...b }), {});

// Cases: [f32|f16]_[exp|whole]
const scalar_cases = ['f32', 'f16', 'abstract'].
flatMap((trait) =>
['exp', 'fract'].map((portion) => ({
  [`${trait}_${portion}`]: () => {
    return FP[trait].
    scalarRange().
    map((v) => portion === 'exp' ? makeCaseExp(v, trait) : makeCaseFract(v, trait));
  }
}))
).
reduce((a, b) => ({ ...a, ...b }), {});

export const d = makeCaseCache('frexp', {
  ...scalar_cases,
  ...vec_cases
});