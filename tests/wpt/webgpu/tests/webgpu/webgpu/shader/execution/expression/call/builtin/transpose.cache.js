/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { FP } from '../../../../../util/floating_point.js';import { makeCaseCache } from '../../case_cache.js';
// Cases: [f32|f16|abstract]_matCxR_[non_]const
// abstract_matCxR_non_const is empty and not used
const cases = ['f32', 'f16', 'abstract'].
flatMap((trait) =>
[2, 3, 4].flatMap((cols) =>
[2, 3, 4].flatMap((rows) =>
[true, false].map((nonConst) => ({
  [`${trait}_mat${cols}x${rows}_${nonConst ? 'non_const' : 'const'}`]: () => {
    if (trait === 'abstract' && nonConst) {
      return [];
    }
    return FP[trait].generateMatrixToMatrixCases(
      FP[trait].sparseMatrixRange(cols, rows),
      nonConst ? 'unfiltered' : 'finite',
      FP[trait].transposeInterval
    );
  }
}))
)
)
).
reduce((a, b) => ({ ...a, ...b }), {});

export const d = makeCaseCache('transpose', cases);