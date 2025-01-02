/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { Float16Array } from '../../external/petamoriken/float16/float16.js'; /**
 * Once-allocated ArrayBuffer/views to avoid overhead of allocation when converting between numeric formats
 *
 * workingData* is shared between multiple functions in this file, so to avoid re-entrancy problems, make sure in
 * functions that use it that they don't call themselves or other functions that use workingData*.
 */
const workingData = new ArrayBuffer(8);
const workingDataU32 = new Uint32Array(workingData);
const workingDataU16 = new Uint16Array(workingData);
const workingDataF32 = new Float32Array(workingData);
const workingDataF16 = new Float16Array(workingData);
const workingDataI32 = new Int32Array(workingData);
const workingDataF64 = new Float64Array(workingData);
const workingDataU64 = new BigUint64Array(workingData);

/**
 * @returns a 64-bit float value via interpreting the input as the bit
 * representation as a 64-bit integer
 */
export function reinterpretU64AsF64(input) {
  workingDataU64[0] = input;
  return workingDataF64[0];
}

/**
 * @returns the 64-bit integer bit representation of 64-bit float value
 */
export function reinterpretF64AsU64(input) {
  workingDataF64[0] = input;
  return workingDataU64[0];
}

// Encoding to u32s, instead of BigInt, for serialization
export function reinterpretF64AsU32s(f64) {
  workingDataF64[0] = f64;
  return [workingDataU32[0], workingDataU32[1]];
}

// De-encoding from u32s, instead of BigInt, for serialization
export function reinterpretU32sAsF64(u32s) {
  workingDataU32[0] = u32s[0];
  workingDataU32[1] = u32s[1];
  return workingDataF64[0];
}

/**
 * @returns a number representing the u32 interpretation
 * of the bits of a number assumed to be an f32 value.
 */
export function reinterpretF32AsU32(f32) {
  workingDataF32[0] = f32;
  return workingDataU32[0];
}

/**
 * @returns a number representing the i32 interpretation
 * of the bits of a number assumed to be an f32 value.
 */
export function reinterpretF32AsI32(f32) {
  workingDataF32[0] = f32;
  return workingDataI32[0];
}

/**
 * @returns a number representing the f32 interpretation
 * of the bits of a number assumed to be an u32 value.
 */
export function reinterpretU32AsF32(u32) {
  workingDataU32[0] = u32;
  return workingDataF32[0];
}

/**
 * @returns a number representing the i32 interpretation
 * of the bits of a number assumed to be an u32 value.
 */
export function reinterpretU32AsI32(u32) {
  workingDataU32[0] = u32;
  return workingDataI32[0];
}

/**
 * @returns a number representing the u32 interpretation
 * of the bits of a number assumed to be an i32 value.
 */
export function reinterpretI32AsU32(i32) {
  workingDataI32[0] = i32;
  return workingDataU32[0];
}

/**
 * @returns a number representing the f32 interpretation
 * of the bits of a number assumed to be an i32 value.
 */
export function reinterpretI32AsF32(i32) {
  workingDataI32[0] = i32;
  return workingDataF32[0];
}

/**
 * @returns a number representing the u16 interpretation
 * of the bits of a number assumed to be an f16 value.
 */
export function reinterpretF16AsU16(f16) {
  workingDataF16[0] = f16;
  return workingDataU16[0];
}

/**
 * @returns a number representing the f16 interpretation
 * of the bits of a number assumed to be an u16 value.
 */
export function reinterpretU16AsF16(u16) {
  workingDataU16[0] = u16;
  return workingDataF16[0];
}