/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { i32 } from '../../../../../util/conversion.js';import { FP } from '../../../../../util/floating_point.js';import { fullI32Range } from '../../../../../util/math.js';
import { makeCaseCache } from '../../case_cache.js';

// Cases: [f32|f16|abstract]
const fp_cases = ['f32', 'f16', 'abstract'].
map((trait) => ({
  [`${trait}`]: () => {
    return FP[trait].generateScalarToIntervalCases(
      FP[trait].scalarRange(),
      'unfiltered',
      FP[trait].signInterval
    );
  }
})).
reduce((a, b) => ({ ...a, ...b }), {});

export const d = makeCaseCache('sign', {
  ...fp_cases,
  i32: () =>
  fullI32Range().map((i) => {
    const signFunc = (i) => i < 0 ? -1 : i > 0 ? 1 : 0;
    return { input: [i32(i)], expected: i32(signFunc(i)) };
  })
});