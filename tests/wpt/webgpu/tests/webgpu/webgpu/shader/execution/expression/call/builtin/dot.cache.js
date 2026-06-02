/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { assert } from '../../../../../../common/util/util.js';import { kValue } from '../../../../../util/constants.js';
import { FP } from '../../../../../util/floating_point.js';
import {
  calculatePermutations,
  sparseVectorI32Range,
  sparseVectorI64Range,
  sparseVectorU32Range,
  vectorI32Range,
  vectorI64Range,
  vectorU32Range } from
'../../../../../util/math.js';
import {
  generateVectorVectorToI32Cases,
  generateVectorVectorToI64Cases,
  generateVectorVectorToU32Cases } from
'../../case.js';
import { makeCaseCache } from '../../case_cache.js';

function ai_dot(x, y) {
  assert(x.length === y.length, 'Cannot calculate dot for vectors of different lengths');
  const multiplications = x.map((_, idx) => x[idx] * y[idx]);
  if (multiplications.some(kValue.i64.isOOB)) return undefined;

  const result = multiplications.reduce((prev, curr) => prev + curr);
  if (kValue.i64.isOOB(result)) return undefined;

  // The spec does not state the ordering of summation, so all the
  // permutations are calculated and the intermediate results checked for
  // going OOB. vec2 does not need permutations, since a + b === b + a.
  // All the end results should be the same regardless of the order if the
  // intermediate additions stay inbounds.
  if (x.length !== 2) {
    let wentOOB = false;
    const permutations = calculatePermutations(multiplications);
    permutations.forEach((p) => {
      if (!wentOOB) {
        p.reduce((prev, curr) => {
          const next = prev + curr;
          if (kValue.i64.isOOB(next)) {
            wentOOB = true;
          }
          return next;
        });
      }
    });

    if (wentOOB) return undefined;
  }

  return !kValue.i64.isOOB(result) ? result : undefined;
}

function ci_dot(x, y) {
  assert(x.length === y.length, 'Cannot calculate dot for vectors of different lengths');
  return x.reduce((prev, _, idx) => prev + Math.imul(x[idx], y[idx]), 0);
}

// Cases: [f32|f16|abstract]_vecN_[non_]const
const float_cases = ['f32', 'f16', 'abstract'].
flatMap((trait) =>
[2, 3, 4].flatMap((N) =>
[true, false].map((nonConst) => ({
  [`${trait === 'abstract' ? 'abstract_float' : trait}_vec${N}_${
  nonConst ? 'non_const' : 'const'
  }`]: () => {
    // Emit an empty array for not const abstract float, since they will never be run
    if (trait === 'abstract' && nonConst) {
      return [];
    }
    // vec3 and vec4 require calculating all possible permutations, so their runtime is much
    // longer per test, so only using sparse vectors for them.
    return FP[trait].generateVectorPairToIntervalCases(
      N === 2 ? FP[trait].vectorRange(2) : FP[trait].sparseVectorRange(N),
      N === 2 ? FP[trait].vectorRange(2) : FP[trait].sparseVectorRange(N),
      nonConst ? 'unfiltered' : 'finite',
      // dot has an inherited accuracy, so abstract is only expected to be as accurate as f32
      FP[trait !== 'abstract' ? trait : 'f32'].dotInterval
    );
  }
}))
)
).
reduce((a, b) => ({ ...a, ...b }), {});

const cases = {
  ...float_cases,
  abstract_int_vec2: () => {
    return generateVectorVectorToI64Cases(vectorI64Range(2), vectorI64Range(2), ai_dot);
  },
  abstract_int_vec3: () => {
    return generateVectorVectorToI64Cases(sparseVectorI64Range(3), sparseVectorI64Range(3), ai_dot);
  },
  abstract_int_vec4: () => {
    return generateVectorVectorToI64Cases(sparseVectorI64Range(4), sparseVectorI64Range(4), ai_dot);
  },
  i32_vec2: () => {
    return generateVectorVectorToI32Cases(vectorI32Range(2), vectorI32Range(2), ci_dot);
  },
  i32_vec3: () => {
    return generateVectorVectorToI32Cases(sparseVectorI32Range(3), sparseVectorI32Range(3), ci_dot);
  },
  i32_vec4: () => {
    return generateVectorVectorToI32Cases(sparseVectorI32Range(4), sparseVectorI32Range(4), ci_dot);
  },
  u32_vec2: () => {
    return generateVectorVectorToU32Cases(vectorU32Range(2), vectorU32Range(2), ci_dot);
  },
  u32_vec3: () => {
    return generateVectorVectorToU32Cases(sparseVectorU32Range(3), sparseVectorU32Range(3), ci_dot);
  },
  u32_vec4: () => {
    return generateVectorVectorToU32Cases(sparseVectorU32Range(4), sparseVectorU32Range(4), ci_dot);
  }
};

export const d = makeCaseCache('dot', cases);