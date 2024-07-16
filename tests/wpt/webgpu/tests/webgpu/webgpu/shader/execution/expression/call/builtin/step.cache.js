/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { anyOf } from '../../../../../util/compare.js';import { FP } from '../../../../../util/floating_point.js';
import { makeCaseCache } from '../../case_cache.js';

// stepInterval's return value can't always be interpreted as a single acceptance
// interval, valid result may be 0.0 or 1.0 or both of them, but will never be a
// value in interval (0.0, 1.0).
// See the comment block on stepInterval for more details
const makeCase = (trait, edge, x) => {
  const FPTrait = FP[trait];
  edge = FPTrait.quantize(edge);
  x = FPTrait.quantize(x);
  const expected = FPTrait.stepInterval(edge, x);

  // [0, 0], [1, 1], or [-∞, +∞] cases
  if (expected.isPoint() || !expected.isFinite()) {
    return { input: [FPTrait.scalarBuilder(edge), FPTrait.scalarBuilder(x)], expected };
  }

  // [0, 1] case, valid result is either 0.0 or 1.0.
  const zeroInterval = FPTrait.toInterval(0);
  const oneInterval = FPTrait.toInterval(1);
  return {
    input: [FPTrait.scalarBuilder(edge), FPTrait.scalarBuilder(x)],
    expected: anyOf(zeroInterval, oneInterval)
  };
};

// Cases: [f32|f16|abstract]
const cases = ['f32', 'f16', 'abstract'].
map((trait) => ({
  [`${trait}`]: () => {
    return FP[trait].
    sparseScalarRange().
    flatMap((edge) => FP[trait].sparseScalarRange().map((x) => makeCase(trait, edge, x)));
  }
})).
reduce((a, b) => ({ ...a, ...b }), {});

export const d = makeCaseCache('step', cases);