/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { toVector } from '../../../../../util/conversion.js';import { FP } from '../../../../../util/floating_point.js';

import { makeCaseCache } from '../../case_cache.js';


// Using a bespoke implementation of make*Case and generate*Cases here
// since refract is the only builtin with the API signature
// (vec, vec, scalar) -> vec

/**
 * @returns a Case for `refract`
 * @param kind what type of floating point numbers to operate on
 * @param i the `i` param for the case
 * @param s the `s` param for the case
 * @param r the `r` param for the case
 * @param check what interval checking to apply
 * */
function makeCase(
kind,
i,
s,
r,
check)
{
  const fp = FP[kind];
  i = i.map(fp.quantize);
  s = s.map(fp.quantize);
  r = fp.quantize(r);

  const vectors = fp.refractInterval(i, s, r);
  if (check === 'finite' && vectors.some((e) => !e.isFinite())) {
    return undefined;
  }

  return {
    input: [toVector(i, fp.scalarBuilder), toVector(s, fp.scalarBuilder), fp.scalarBuilder(r)],
    expected: fp.refractInterval(i, s, r)
  };
}

/**
 * @returns an array of Cases for `refract`
 * @param kind what type of floating point numbers to operate on
 * @param param_is array of inputs to try for the `i` param
 * @param param_ss array of inputs to try for the `s` param
 * @param param_rs array of inputs to try for the `r` param
 * @param check what interval checking to apply
 */
function generateCases(
kind,
param_is,
param_ss,
param_rs,
check)
{
  // Cannot use `cartesianProduct` here due to heterogeneous param types
  return param_is.
  flatMap((i) => {
    return param_ss.flatMap((s) => {
      return param_rs.map((r) => {
        return makeCase(kind, i, s, r, check);
      });
    });
  }).
  filter((c) => c !== undefined);
}

// Cases: [f32|f16]_vecN_[non_]const
const cases = ['f32', 'f16'].
flatMap((trait) =>
[2, 3, 4].flatMap((dim) =>
[true, false].map((nonConst) => ({
  [`${trait}_vec${dim}_${nonConst ? 'non_const' : 'const'}`]: () => {
    return generateCases(
      trait,
      FP[trait].sparseVectorRange(dim),
      FP[trait].sparseVectorRange(dim),
      FP[trait].sparseScalarRange(),
      nonConst ? 'unfiltered' : 'finite'
    );
  }
}))
)
).
reduce((a, b) => ({ ...a, ...b }), {});

export const d = makeCaseCache('refract', cases);