/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `Special and sample values for WGSL scalar types`;import { assert } from '../../common/util/util.js';
import { uint32ToFloat32 } from '../util/conversion.js';

/** Returns an array of subnormal f32 numbers.
 * Subnormals are non-zero finite numbers with the minimum representable
 * exponent.
 */
export function subnormalF32Examples() {
  // The results, as uint32 values.
  const result_as_bits = [];

  const max_mantissa = 0x7f_ffff;
  const sign_bits = [0, 0x8000_0000];
  for (const sign_bit of sign_bits) {
    // exponent bits must be zero.
    const sign_and_exponent = sign_bit;

    // Set all bits
    result_as_bits.push(sign_and_exponent | max_mantissa);

    // Set each of the lower bits individually.
    for (let lower_bits = 1; lower_bits <= max_mantissa; lower_bits <<= 1) {
      result_as_bits.push(sign_and_exponent | lower_bits);
    }
  }
  assert(
    result_as_bits.length === 2 * (1 + 23),
    'subnormal number sample count is ' + result_as_bits.length.toString()
  );
  return result_as_bits.map((u) => uint32ToFloat32(u));
}

/** Returns an array of normal f32 numbers.
 * Normal numbers are not: zero, Nan, infinity, subnormal.
 */
export function normalF32Examples() {
  const result = [1.0, -2.0];

  const max_mantissa_as_bits = 0x7f_ffff;
  const min_exponent_as_bits = 0x0080_0000;
  const max_exponent_as_bits = 0x7f00_0000; // Max normal exponent
  const sign_bits = [0, 0x8000_0000];
  for (const sign_bit of sign_bits) {
    for (let e = min_exponent_as_bits; e <= max_exponent_as_bits; e += min_exponent_as_bits) {
      const sign_and_exponent = sign_bit | e;

      // Set zero mantissa bits
      result.push(uint32ToFloat32(sign_and_exponent));
      // Set all mantissa bits
      result.push(uint32ToFloat32(sign_and_exponent | max_mantissa_as_bits));

      // Set each of the lower bits individually.
      for (let lower_bits = 1; lower_bits <= max_mantissa_as_bits; lower_bits <<= 1) {
        result.push(uint32ToFloat32(sign_and_exponent | lower_bits));
      }
    }
  }
  assert(
    result.length === 2 + 2 * 254 * 25,
    'normal number sample count is ' + result.length.toString()
  );
  return result;
}

/** Returns an array of 32-bit NaNs, as Uint32 bit patterns.
 * NaNs have: maximum exponent, but the mantissa is not zero.
 */
export function nanF32BitsExamples() {
  const result = [];
  const exponent_bit = 0x7f80_0000;
  const sign_bits = [0, 0x8000_0000];
  for (const sign_bit of sign_bits) {
    const sign_and_exponent = sign_bit | exponent_bit;
    const bits = sign_and_exponent | 0x40_0000;
    // Only the most significant bit of the mantissa is set.
    result.push(bits);

    // Quiet and signalling NaNs differ based on the most significant bit
    // of the mantissa. Try both.
    for (const quiet_signalling of [0, 0x40_0000]) {
      // Set each of the lower bits.
      for (let lower_bits = 1; lower_bits < 0x40_0000; lower_bits <<= 1) {
        const bits = sign_and_exponent | quiet_signalling | lower_bits;
        result.push(bits);
      }
    }
  }
  return result;
}