/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { toVector } from '../../../../../util/conversion.js';import { FP } from '../../../../../util/floating_point.js';
import { selectNCases } from '../../case.js';
import { makeCaseCache } from '../../case_cache.js';


// Using a bespoke implementation of make*Case and generate*Cases here
// since refract is the only builtin with the API signature
// (vec, vec, scalar) -> vec

/**
 * @returns a Case for `refract`
 * @param argumentKind what kind of floating point numbers being operated on
 * @param parameterKind what kind of floating point operation should be performed,
 *                should be the same as argumentKind, except for abstract
 * @param i the `i` param for the case
 * @param s the `s` param for the case
 * @param r the `r` param for the case
 * @param check what interval checking to apply
 * */
function makeCase(
argumentKind,
parameterKind,
i,
s,
r,
check)
{
  const fp = FP[argumentKind];
  i = i.map(fp.quantize);
  s = s.map(fp.quantize);
  r = fp.quantize(r);

  const vectors = FP[parameterKind].refractInterval(i, s, r);
  if (check === 'finite' && vectors.some((e) => !e.isFinite())) {
    return undefined;
  }

  return {
    input: [toVector(i, fp.scalarBuilder), toVector(s, fp.scalarBuilder), fp.scalarBuilder(r)],
    expected: vectors
  };
}

/**
 * @returns an array of Cases for `refract`
 * @param argumentKind what kind of floating point numbers being operated on
 * @param parameterKind what kind of floating point operation should be performed,
 *                should be the same as argumentKind, except for abstract
 * @param param_is array of inputs to try for the `i` param
 * @param param_ss array of inputs to try for the `s` param
 * @param param_rs array of inputs to try for the `r` param
 * @param check what interval checking to apply
 */
function generateCases(
argumentKind,
parameterKind,
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
        return makeCase(argumentKind, parameterKind, i, s, r, check);
      });
    });
  }).
  filter((c) => c !== undefined);
}

// Cases: [f32|f16|abstract]_vecN_[non_]const
const cases = ['f32', 'f16', 'abstract'].
flatMap((trait) =>
[2, 3, 4].flatMap((dim) =>
[true, false].map((nonConst) => ({
  [`${trait}_vec${dim}_${nonConst ? 'non_const' : 'const'}`]: () => {
    if (trait === 'abstract' && nonConst) {
      return [];
    }
    if (trait !== 'abstract') {
      return generateCases(
        trait,
        trait,
        FP[trait].sparseVectorRange(dim),
        FP[trait].sparseVectorRange(dim),
        FP[trait].sparseScalarRange(),
        nonConst ? 'unfiltered' : 'finite'
      );
    } else {
      // Restricting the number of cases, because a vector of abstract floats needs to be returned, which is costly.
      return selectNCases(
        'faceForward',
        20,
        generateCases(
          trait,
          // refract has an inherited accuracy, so is only expected to be as accurate as f32
          'f32',
          FP[trait].sparseVectorRange(dim),
          FP[trait].sparseVectorRange(dim),
          FP[trait].sparseScalarRange(),
          nonConst ? 'unfiltered' : 'finite'
        )
      );
    }
  }
}))
)
).
reduce((a, b) => ({ ...a, ...b }), {});

export const d = makeCaseCache('refract', cases);