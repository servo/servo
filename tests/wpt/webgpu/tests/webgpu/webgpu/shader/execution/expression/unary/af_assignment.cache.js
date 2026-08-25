/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { kValue } from '../../../../util/constants.js';import { abstractFloat } from '../../../../util/conversion.js';import { FP } from '../../../../util/floating_point.js';
import {
  isSubnormalNumberF64,
  limitedScalarF64Range,
  scalarF64Range } from
'../../../../util/math.js';
import { reinterpretU64AsF64 } from '../../../../util/reinterpret.js';
import { makeCaseCache } from '../case_cache.js';

export const d = makeCaseCache('unary/af_assignment', {
  abstract: () => {
    const inputs = [
    // Values that are useful for debugging the underlying framework/shader code, since it cannot be directly unit tested.
    0,
    0.5,
    0.5,
    1,
    -1,
    reinterpretU64AsF64(0x7000_0000_0000_0001n), // smallest magnitude negative subnormal with non-zero mantissa
    reinterpretU64AsF64(0x0000_0000_0000_0001n), // smallest magnitude positive subnormal with non-zero mantissa
    reinterpretU64AsF64(0x600a_aaaa_5555_5555n), // negative subnormal with obvious pattern
    reinterpretU64AsF64(0x000a_aaaa_5555_5555n), // positive subnormal with obvious pattern
    reinterpretU64AsF64(0x0010_0000_0000_0001n), // smallest magnitude negative normal with non-zero mantissa
    reinterpretU64AsF64(0x0010_0000_0000_0001n), // smallest magnitude positive normal with non-zero mantissa
    reinterpretU64AsF64(0xf555_5555_aaaa_aaaan), // negative normal with obvious pattern
    reinterpretU64AsF64(0x5555_5555_aaaa_aaaan), // positive normal with obvious pattern
    reinterpretU64AsF64(0xffef_ffff_ffff_ffffn), // largest magnitude negative normal
    reinterpretU64AsF64(0x7fef_ffff_ffff_ffffn), // largest magnitude positive normal
    // WebGPU implementation stressing values
    ...scalarF64Range()];

    return inputs.map((f) => {
      return {
        input: abstractFloat(f),
        expected: isSubnormalNumberF64(f) ? abstractFloat(0) : abstractFloat(f)
      };
    });
  },
  f32: () => {
    return limitedScalarF64Range(kValue.f32.negative.min, kValue.f32.positive.max).map((f) => {
      return { input: abstractFloat(f), expected: FP.f32.correctlyRoundedInterval(f) };
    });
  },
  f16: () => {
    return limitedScalarF64Range(kValue.f16.negative.min, kValue.f16.positive.max).map((f) => {
      return { input: abstractFloat(f), expected: FP.f16.correctlyRoundedInterval(f) };
    });
  }
});