/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { kValue } from '../../../../../util/constants.js';import { FP } from '../../../../../util/floating_point.js';import { biasedRange, linearRange } from '../../../../../util/math.js';
import { makeCaseCache } from '../../case_cache.js';

// floor(log2(max f32 value)) = 127, so exp2(127) will be within range of a f32, but exp2(128) will not
// floor(ln(max f64 value)) = 1023, so exp2(1023) can be handled by the testing framework, but exp2(1024) will misbehave
const f32_inputs = [
0, // Returns 1 by definition
-128, // Returns subnormal value
kValue.f32.negative.min, // Closest to returning 0 as possible
...biasedRange(kValue.f32.negative.max, -127, 100),
...biasedRange(kValue.f32.positive.min, 127, 100),
...linearRange(128, 1023, 10) // Overflows f32, but not f64
];

// floor(log2(max f16 value)) = 15, so exp2(15) will be within range of a f16, but exp2(15) will not
const f16_inputs = [
0, // Returns 1 by definition
-16, // Returns subnormal value
kValue.f16.negative.min, // Closest to returning 0 as possible
...biasedRange(kValue.f16.negative.max, -15, 100),
...biasedRange(kValue.f16.positive.min, 15, 100),
...linearRange(16, 1023, 10) // Overflows f16, but not f64
];

export const d = makeCaseCache('exp2', {
  f32_const: () => {
    return FP.f32.generateScalarToIntervalCases(f32_inputs, 'finite', FP.f32.exp2Interval);
  },
  f32_non_const: () => {
    return FP.f32.generateScalarToIntervalCases(f32_inputs, 'unfiltered', FP.f32.exp2Interval);
  },
  f16_const: () => {
    return FP.f16.generateScalarToIntervalCases(f16_inputs, 'finite', FP.f16.exp2Interval);
  },
  f16_non_const: () => {
    return FP.f16.generateScalarToIntervalCases(f16_inputs, 'unfiltered', FP.f16.exp2Interval);
  },
  abstract: () => {
    // exp2 has an ulp accuracy, so is only expected to be as accurate as f32
    return FP.abstract.generateScalarToIntervalCases(f32_inputs, 'finite', FP.f32.exp2Interval);
  }
});