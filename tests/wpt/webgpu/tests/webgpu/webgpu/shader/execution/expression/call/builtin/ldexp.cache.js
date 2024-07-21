/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { assert } from '../../../../../../common/util/util.js';import { anyOf } from '../../../../../util/compare.js';import { abstractInt, i32 } from '../../../../../util/conversion.js';
import { FP } from '../../../../../util/floating_point.js';
import { biasedRange, quantizeToI32, sparseI32Range } from '../../../../../util/math.js';

import { makeCaseCache } from '../../case_cache.js';

// ldexpInterval's return interval doesn't cover the flush-to-zero cases when e2 + bias <= 0, thus
// special examination is required.
// See the comment block on ldexpInterval for more details
// e2 is an integer (i32) while e1 is float.
const makeCase = (trait, e1, e2) => {
  const FPTrait = FP[trait];
  e1 = FPTrait.quantize(e1);
  // e2 should be in i32 range for the convenience.
  assert(-2147483648 <= e2 && e2 <= 2147483647, 'e2 should be in i32 range');
  e2 = quantizeToI32(e2);

  const expected = FPTrait.ldexpInterval(e1, e2);

  const e2_scalar = trait === 'abstract' ? abstractInt(BigInt(e2)) : i32(e2);
  // Result may be zero if e2 + bias <= 0
  if (e2 + FPTrait.constants().bias <= 0) {
    return {
      input: [FPTrait.scalarBuilder(e1), e2_scalar],
      expected: anyOf(expected, FPTrait.constants().zeroInterval)
    };
  }

  return { input: [FPTrait.scalarBuilder(e1), e2_scalar], expected };
};

// Cases: [f32|f16|abstract]_[non_]const
const cases = ['f32', 'f16', 'abstract'].
flatMap((trait) =>
[true, false].map((nonConst) => ({
  [`${trait}_${nonConst ? 'non_const' : 'const'}`]: () => {
    if (nonConst) {
      if (trait === 'abstract') {
        return [];
      }
      return FP[trait].
      sparseScalarRange().
      flatMap((e1) => sparseI32Range().map((e2) => makeCase(trait, e1, e2)));
    }
    const bias = FP[trait].constants().bias;
    // const
    return FP[trait].
    sparseScalarRange().
    flatMap((e1) =>
    biasedRange(-bias - 10, bias + 1, 10).flatMap((e2) =>
    FP[trait].isFinite(e1 * 2 ** quantizeToI32(e2)) ? makeCase(trait, e1, e2) : []
    )
    );
  }
}))
).
reduce((a, b) => ({ ...a, ...b }), {});

export const d = makeCaseCache('ldexp', cases);