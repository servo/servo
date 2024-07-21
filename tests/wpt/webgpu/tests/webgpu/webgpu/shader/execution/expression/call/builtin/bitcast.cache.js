/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { assert } from '../../../../../../common/util/util.js';import { alwaysPass, anyOf } from '../../../../../util/compare.js';import { kBit, kValue } from '../../../../../util/constants.js';
import {

  VectorValue,
  f16,
  f32,
  i32,
  toVector,
  u32,
  abstractFloat,
  abstractInt } from
'../../../../../util/conversion.js';
import { FP, FPInterval } from '../../../../../util/floating_point.js';
import {
  cartesianProduct,
  fullI32Range,
  fullU32Range,
  isFiniteF16,
  isFiniteF32,
  isSubnormalNumberF16,
  isSubnormalNumberF32,
  linearRange,
  scalarF16Range,
  scalarF32Range } from
'../../../../../util/math.js';
import {
  reinterpretF16AsU16,
  reinterpretF32AsI32,
  reinterpretF32AsU32,
  reinterpretI32AsF32,
  reinterpretI32AsU32,
  reinterpretU16AsF16,
  reinterpretU32AsF32,
  reinterpretU32AsI32 } from
'../../../../../util/reinterpret.js';
import { makeCaseCache } from '../../case_cache.js';

const numNaNs = 11;
const f32InfAndNaNInU32 = [
// Cover NaNs evenly in integer space.
// The positive NaN with the lowest integer representation is the integer
// for infinity, plus one.
// The positive NaN with the highest integer representation is i32.max (!)
...linearRange(kBit.f32.positive.infinity + 1, kBit.i32.positive.max, numNaNs),
// The negative NaN with the lowest integer representation is the integer
// for negative infinity, plus one.
// The negative NaN with the highest integer representation is u32.max (!)
...linearRange(kBit.f32.negative.infinity + 1, kBit.u32.max, numNaNs),
kBit.f32.positive.infinity,
kBit.f32.negative.infinity];

const f32InfAndNaNInF32 = f32InfAndNaNInU32.map((u) => reinterpretU32AsF32(u));
const f32InfAndNaNInI32 = f32InfAndNaNInU32.map((u) => reinterpretU32AsI32(u));

const f32ZerosInU32 = [0, kBit.f32.negative.zero];
const f32ZerosInF32 = f32ZerosInU32.map((u) => reinterpretU32AsF32(u));
const f32ZerosInI32 = f32ZerosInU32.map((u) => reinterpretU32AsI32(u));
const f32ZerosInterval = new FPInterval('f32', -0.0, 0.0);

// f32FiniteRange is a list of finite f32s. fullF32Range() already
// has +0, we only need to add -0.
const f32FiniteRange = [...scalarF32Range(), kValue.f32.negative.zero];
const f32RangeWithInfAndNaN = [...f32FiniteRange, ...f32InfAndNaNInF32];

// Type.f16 values, finite, Inf/NaN, and zeros. Represented in float and u16.
const f16FiniteInF16 = [...scalarF16Range(), kValue.f16.negative.zero];
const f16FiniteInU16 = f16FiniteInF16.map((u) => reinterpretF16AsU16(u));

const f16InfAndNaNInU16 = [
// Cover NaNs evenly in integer space.
// The positive NaN with the lowest integer representation is the integer
// for infinity, plus one.
// The positive NaN with the highest integer representation is u16 0x7fff i.e. 32767.
...linearRange(kBit.f16.positive.infinity + 1, 32767, numNaNs).map((v) => Math.ceil(v)),
// The negative NaN with the lowest integer representation is the integer
// for negative infinity, plus one.
// The negative NaN with the highest integer representation is u16 0xffff i.e. 65535
...linearRange(kBit.f16.negative.infinity + 1, 65535, numNaNs).map((v) => Math.floor(v)),
kBit.f16.positive.infinity,
kBit.f16.negative.infinity];

const f16InfAndNaNInF16 = f16InfAndNaNInU16.map((u) => reinterpretU16AsF16(u));

const f16ZerosInU16 = [kBit.f16.negative.zero, 0];

// f16 interval that match +/-0.0.
const f16ZerosInterval = new FPInterval('f16', -0.0, 0.0);

/**
 * @returns an u32 whose lower and higher 16bits are the two elements of the
 * given array of two u16 respectively, in little-endian.
 */
function u16x2ToU32(u16x2) {
  assert(u16x2.length === 2);
  // Create a DataView with 4 bytes buffer.
  const buffer = new ArrayBuffer(4);
  const view = new DataView(buffer);
  // Enforce little-endian.
  view.setUint16(0, u16x2[0], true);
  view.setUint16(2, u16x2[1], true);
  return view.getUint32(0, true);
}

/**
 * @returns an array of two u16, respectively the lower and higher 16bits of
 * given u32 in little-endian.
 */
function u32ToU16x2(u32) {
  // Create a DataView with 4 bytes buffer.
  const buffer = new ArrayBuffer(4);
  const view = new DataView(buffer);
  // Enforce little-endian.
  view.setUint32(0, u32, true);
  return [view.getUint16(0, true), view.getUint16(2, true)];
}

/**
 * @returns a vec2<f16> from an array of two u16, each reinterpreted as f16.
 */
function u16x2ToVec2F16(u16x2) {
  assert(u16x2.length === 2);
  return toVector(u16x2.map(reinterpretU16AsF16), f16);
}

/**
 * @returns a vec4<f16> from an array of four u16, each reinterpreted as f16.
 */
function u16x4ToVec4F16(u16x4) {
  assert(u16x4.length === 4);
  return toVector(u16x4.map(reinterpretU16AsF16), f16);
}

/**
 * @returns true if and only if a given u32 can bitcast to a vec2<f16> with all elements
 * being finite f16 values.
 */
function canU32BitcastToFiniteVec2F16(u32) {
  return u32ToU16x2(u32).
  map((u16) => isFiniteF16(reinterpretU16AsF16(u16))).
  reduce((a, b) => a && b, true);
}

/**
 * @returns an array of N elements with the i-th element being an array of len elements
 * [a_i, a_((i+1)%N), ..., a_((i+len-1)%N)], for the input array of N element [a_1, ... a_N]
 * and the given len. For example, slidingSlice([1, 2, 3], 2) result in
 * [[1, 2], [2, 3], [3, 1]].
 * This helper function is used for generating vector cases from scalar values array.
 */
function slidingSlice(input, len) {
  const result = [];
  for (let i = 0; i < input.length; i++) {
    const sub = [];
    for (let j = 0; j < len; j++) {
      sub.push(input[(i + j) % input.length]);
    }
    result.push(sub);
  }
  return result;
}

// vec2<f16> interesting (zeros, Inf, and NaN) values for testing cases.
// vec2<f16> values that has at least one Inf/NaN f16 element, reinterpreted as u32/i32.
const f16Vec2InfAndNaNInU32 = [
...cartesianProduct(f16InfAndNaNInU16, [...f16InfAndNaNInU16, ...f16FiniteInU16]),
...cartesianProduct(f16FiniteInU16, f16InfAndNaNInU16)].
map(u16x2ToU32);
const f16Vec2InfAndNaNInI32 = f16Vec2InfAndNaNInU32.map((u) => reinterpretU32AsI32(u));
// vec2<f16> values with two f16 0.0 element, reinterpreted as u32/i32.
const f16Vec2ZerosInU32 = cartesianProduct(f16ZerosInU16, f16ZerosInU16).map(u16x2ToU32);
const f16Vec2ZerosInI32 = f16Vec2ZerosInU32.map((u) => reinterpretU32AsI32(u));

// i32/u32/f32 range for bitcasting to vec2<f16>
// u32 values for bitcasting to vec2<f16> finite, Inf, and NaN.
const u32RangeForF16Vec2FiniteInfNaN = [
...fullU32Range(),
...f16Vec2ZerosInU32,
...f16Vec2InfAndNaNInU32];

// u32 values for bitcasting to finite only vec2<f16>, used for constant evaluation.
const u32RangeForF16Vec2Finite = u32RangeForF16Vec2FiniteInfNaN.filter(
  canU32BitcastToFiniteVec2F16
);
// i32 values for bitcasting to vec2<f16> finite, zeros, Inf, and NaN.
const i32RangeForF16Vec2FiniteInfNaN = [
...fullI32Range(),
...f16Vec2ZerosInI32,
...f16Vec2InfAndNaNInI32];

// i32 values for bitcasting to finite only vec2<f16>, used for constant evaluation.
const i32RangeForF16Vec2Finite = i32RangeForF16Vec2FiniteInfNaN.filter((u) =>
canU32BitcastToFiniteVec2F16(reinterpretI32AsU32(u))
);
// f32 values with finite/Inf/NaN f32, for bitcasting to vec2<f16> finite, zeros, Inf, and NaN.
const f32RangeWithInfAndNaNForF16Vec2FiniteInfNaN = [
...f32RangeWithInfAndNaN,
...u32RangeForF16Vec2FiniteInfNaN.map(reinterpretU32AsF32)];

// Finite f32 values for bitcasting to finite only vec2<f16>, used for constant evaluation.
const f32FiniteRangeForF16Vec2Finite = f32RangeWithInfAndNaNForF16Vec2FiniteInfNaN.
filter(isFiniteF32).
filter((u) => canU32BitcastToFiniteVec2F16(reinterpretF32AsU32(u)));

// vec2<f16> cases for bitcasting to i32/u32/f32, by combining f16 values into pairs
const f16Vec2FiniteInU16x2 = slidingSlice(f16FiniteInU16, 2);
const f16Vec2FiniteInfNanInU16x2 = slidingSlice([...f16FiniteInU16, ...f16InfAndNaNInU16], 2);
// vec4<f16> cases for bitcasting to vec2<i32/u32/f32>, by combining f16 values 4-by-4
const f16Vec2FiniteInU16x4 = slidingSlice(f16FiniteInU16, 4);
const f16Vec2FiniteInfNanInU16x4 = slidingSlice([...f16FiniteInU16, ...f16InfAndNaNInU16], 4);

// alwaysPass comparator for i32/u32/f32 cases. For f32/f16 we also use unbound interval, which
// allow per-element unbounded expectation for vector.
const anyF32 = alwaysPass('any f32');
const anyI32 = alwaysPass('any i32');
const anyU32 = alwaysPass('any u32');

// Unbounded FPInterval
const f32UnboundedInterval = FP.f32.constants().unboundedInterval;
const f16UnboundedInterval = FP.f16.constants().unboundedInterval;

// i32 and u32 cases for bitcasting to f32.
// i32 cases for bitcasting to f32 finite, zeros, Inf, and NaN.
const i32RangeForF32FiniteInfNaN = [
...fullI32Range(),
...f32ZerosInI32,
...f32InfAndNaNInI32];

// i32 cases for bitcasting to f32 finite only.
const i32RangeForF32Finite = i32RangeForF32FiniteInfNaN.filter((i) =>
isFiniteF32(reinterpretI32AsF32(i))
);
// u32 cases for bitcasting to f32 finite, zeros, Inf, and NaN.
const u32RangeForF32FiniteInfNaN = [
...fullU32Range(),
...f32ZerosInU32,
...f32InfAndNaNInU32];

// u32 cases for bitcasting to f32 finite only.
const u32RangeForF32Finite = u32RangeForF32FiniteInfNaN.filter((u) =>
isFiniteF32(reinterpretU32AsF32(u))
);

/**
 * @returns a Comparator for checking if a f32 value is a valid
 * bitcast conversion from f32.
 */
function bitcastF32ToF32Comparator(f) {
  if (!isFiniteF32(f)) return anyF32;
  const acceptable = [f, ...(isSubnormalNumberF32(f) ? f32ZerosInF32 : [])];
  return anyOf(...acceptable.map(f32));
}

/**
 * @returns a Comparator for checking if a u32 value is a valid
 * bitcast conversion from f32.
 */
function bitcastF32ToU32Comparator(f) {
  if (!isFiniteF32(f)) return anyU32;
  const acceptable = [
  reinterpretF32AsU32(f),
  ...(isSubnormalNumberF32(f) ? f32ZerosInU32 : [])];

  return anyOf(...acceptable.map(u32));
}

/**
 * @returns a Comparator for checking if a i32 value is a valid
 * bitcast conversion from f32.
 */
function bitcastF32ToI32Comparator(f) {
  if (!isFiniteF32(f)) return anyI32;
  const acceptable = [
  reinterpretF32AsI32(f),
  ...(isSubnormalNumberF32(f) ? f32ZerosInI32 : [])];

  return anyOf(...acceptable.map(i32));
}

/**
 * @returns a Comparator for checking if a f32 value is a valid
 * bitcast conversion from i32.
 */
function bitcastI32ToF32Comparator(i) {
  const f = reinterpretI32AsF32(i);
  if (!isFiniteF32(f)) return anyI32;
  // Positive or negative zero bit pattern map to any zero.
  if (f32ZerosInI32.includes(i)) return anyOf(...f32ZerosInF32.map(f32));
  const acceptable = [f, ...(isSubnormalNumberF32(f) ? f32ZerosInF32 : [])];
  return anyOf(...acceptable.map(f32));
}

/**
 * @returns a Comparator for checking if a f32 value is a valid
 * bitcast conversion from u32.
 */
function bitcastU32ToF32Comparator(u) {
  const f = reinterpretU32AsF32(u);
  if (!isFiniteF32(f)) return anyU32;
  // Positive or negative zero bit pattern map to any zero.
  if (f32ZerosInU32.includes(u)) return anyOf(...f32ZerosInF32.map(f32));
  const acceptable = [f, ...(isSubnormalNumberF32(f) ? f32ZerosInF32 : [])];
  return anyOf(...acceptable.map(f32));
}

/**
 * @returns an array of expected f16 FPInterval for the given bitcasted f16 value, which may be
 * subnormal, Inf, or NaN. Test cases that bitcasted to vector of f16 use this function to get
 * per-element expectation and build vector expectation using cartesianProduct.
 */
function generateF16ExpectationIntervals(bitcastedF16Value) {
  // If the bitcasted f16 value is inf or nan, the result is unbounded
  if (!isFiniteF16(bitcastedF16Value)) {
    return [f16UnboundedInterval];
  }
  // If the casted f16 value is +/-0.0, the result can be one of both. Note that in JS -0.0 === 0.0.
  if (bitcastedF16Value === 0.0) {
    return [f16ZerosInterval];
  }
  const exactInterval = FP.f16.toInterval(bitcastedF16Value);
  // If the casted f16 value is subnormal, it also may be flushed to +/-0.0.
  return [exactInterval, ...(isSubnormalNumberF16(bitcastedF16Value) ? [f16ZerosInterval] : [])];
}

/**
 * @returns a Comparator for checking if a f16 value is a valid
 * bitcast conversion from f16.
 */
function bitcastF16ToF16Comparator(f) {
  if (!isFiniteF16(f)) return anyOf(f16UnboundedInterval);
  return anyOf(...generateF16ExpectationIntervals(f));
}

/**
 * @returns a Comparator for checking if a vec2<f16> is a valid bitcast
 * conversion from u32.
 */
function bitcastU32ToVec2F16Comparator(u) {
  const bitcastedVec2F16InU16x2 = u32ToU16x2(u).map(reinterpretU16AsF16);
  // Generate expection for vec2 f16 result, by generating expected intervals for each elements and
  // then do cartesian product.
  const expectedIntervalsCombination = cartesianProduct(
    ...bitcastedVec2F16InU16x2.map(generateF16ExpectationIntervals)
  );
  return anyOf(...expectedIntervalsCombination);
}

/**
 * @returns a Comparator for checking if a vec2<f16> value is a valid
 * bitcast conversion from i32.
 */
function bitcastI32ToVec2F16Comparator(i) {
  const bitcastedVec2F16InU16x2 = u32ToU16x2(reinterpretI32AsU32(i)).map(reinterpretU16AsF16);
  // Generate expection for vec2 f16 result, by generating expected intervals for each elements and
  // then do cartesian product.
  const expectedIntervalsCombination = cartesianProduct(
    ...bitcastedVec2F16InU16x2.map(generateF16ExpectationIntervals)
  );
  return anyOf(...expectedIntervalsCombination);
}

/**
 * @returns a Comparator for checking if a vec2<f16> value is a valid
 * bitcast conversion from f32.
 */
function bitcastF32ToVec2F16Comparator(f) {
  // If input f32 is not finite, it can be evaluated to any value and thus any result f16 vec2 is
  // possible.
  if (!isFiniteF32(f)) {
    return anyOf([f16UnboundedInterval, f16UnboundedInterval]);
  }
  const bitcastedVec2F16InU16x2 = u32ToU16x2(reinterpretF32AsU32(f)).map(reinterpretU16AsF16);
  // Generate expection for vec2 f16 result, by generating expected intervals for each elements and
  // then do cartesian product.
  const expectedIntervalsCombination = cartesianProduct(
    ...bitcastedVec2F16InU16x2.map(generateF16ExpectationIntervals)
  );
  return anyOf(...expectedIntervalsCombination);
}

/**
 * @returns a Comparator for checking if a vec4<f16> is a valid
 * bitcast conversion from vec2<u32>.
 */
function bitcastVec2U32ToVec4F16Comparator(u32x2) {
  assert(u32x2.length === 2);
  const bitcastedVec4F16InU16x4 = u32x2.flatMap(u32ToU16x2).map(reinterpretU16AsF16);
  // Generate expection for vec4 f16 result, by generating expected intervals for each elements and
  // then do cartesian product.
  const expectedIntervalsCombination = cartesianProduct(
    ...bitcastedVec4F16InU16x4.map(generateF16ExpectationIntervals)
  );
  return anyOf(...expectedIntervalsCombination);
}

/**
 * @returns a Comparator for checking if a vec4<f16> is a valid
 * bitcast conversion from vec2<i32>.
 */
function bitcastVec2I32ToVec4F16Comparator(i32x2) {
  assert(i32x2.length === 2);
  const bitcastedVec4F16InU16x4 = i32x2.
  map(reinterpretI32AsU32).
  flatMap(u32ToU16x2).
  map(reinterpretU16AsF16);
  // Generate expection for vec4 f16 result, by generating expected intervals for each elements and
  // then do cartesian product.
  const expectedIntervalsCombination = cartesianProduct(
    ...bitcastedVec4F16InU16x4.map(generateF16ExpectationIntervals)
  );
  return anyOf(...expectedIntervalsCombination);
}

/**
 * @returns a Comparator for checking if a vec4<f16> is a valid
 * bitcast conversion from vec2<f32>.
 */
function bitcastVec2F32ToVec4F16Comparator(f32x2) {
  assert(f32x2.length === 2);
  const bitcastedVec4F16InU16x4 = f32x2.
  map(reinterpretF32AsU32).
  flatMap(u32ToU16x2).
  map(reinterpretU16AsF16);
  // Generate expection for vec4 f16 result, by generating expected intervals for each elements and
  // then do cartesian product.
  const expectedIntervalsCombination = cartesianProduct(
    ...bitcastedVec4F16InU16x4.map(generateF16ExpectationIntervals)
  );
  return anyOf(...expectedIntervalsCombination);
}

// Structure that store the expectations of a single 32bit scalar/element bitcasted from two f16.







/**
 * @returns the array of possible 16bits, represented in u16, that bitcasted
 * from a given finite f16 represented in u16, handling the possible subnormal
 * flushing. Used to build up 32bits or larger results.
 */
function possibleBitsInU16FromFiniteF16InU16(f16InU16) {
  const h = reinterpretU16AsF16(f16InU16);
  assert(isFiniteF16(h));
  return [f16InU16, ...(isSubnormalNumberF16(h) ? f16ZerosInU16 : [])];
}

/**
 * @returns the expectation for a single 32bit scalar bitcasted from given pair of
 * f16, result in ExpectionFor32BitsScalarFromF16x2.
 */
function possible32BitScalarIntervalsFromF16x2(
f16x2InU16x2,
type)
{
  assert(f16x2InU16x2.length === 2);
  let reinterpretFromU32;
  let expectationsForValue;
  let unboundedExpectations;
  if (type === 'u32') {
    reinterpretFromU32 = (x) => x;
    expectationsForValue = (x) => [u32(x)];
    // Scalar expectation can not express "unbounded" for i32 and u32, so use 0 here as a
    // placeholder, and the possibleExpectations should be ignored if the result is unbounded.
    unboundedExpectations = [u32(0)];
  } else if (type === 'i32') {
    reinterpretFromU32 = (x) => reinterpretU32AsI32(x);
    expectationsForValue = (x) => [i32(x)];
    // Scalar expectation can not express "unbounded" for i32 and u32, so use 0 here as a
    // placeholder, and the possibleExpectations should be ignored if the result is unbounded.
    unboundedExpectations = [i32(0)];
  } else {
    assert(type === 'f32');
    reinterpretFromU32 = (x) => reinterpretU32AsF32(x);
    expectationsForValue = (x) => {
      // Handle the possible Inf/NaN/zeros and subnormal cases for f32 result.
      if (!isFiniteF32(x)) {
        return [f32UnboundedInterval];
      }
      // If the casted f16 value is +/-0.0, the result can be one of both. Note that in JS -0.0 === 0.0.
      if (x === 0.0) {
        return [f32ZerosInterval];
      }
      const exactInterval = FP.f32.toInterval(x);
      // If the casted f16 value is subnormal, it also may be flushed to +/-0.0.
      return [exactInterval, ...(isSubnormalNumberF32(x) ? [f32ZerosInterval] : [])];
    };
    unboundedExpectations = [f32UnboundedInterval];
  }
  // Return unbounded expection if f16 Inf/NaN occurs
  if (
  !isFiniteF16(reinterpretU16AsF16(f16x2InU16x2[0])) ||
  !isFiniteF16(reinterpretU16AsF16(f16x2InU16x2[1])))
  {
    return { possibleExpectations: unboundedExpectations, isUnbounded: true };
  }
  const possibleU16Bits = f16x2InU16x2.map(possibleBitsInU16FromFiniteF16InU16);
  const possibleExpectations = cartesianProduct(...possibleU16Bits).flatMap(

    (possibleBitsU16x2) => {
      assert(possibleBitsU16x2.length === 2);
      return expectationsForValue(reinterpretFromU32(u16x2ToU32(possibleBitsU16x2)));
    });
  return { possibleExpectations, isUnbounded: false };
}

/**
 * @returns a Comparator for checking if a u32 value is a valid
 * bitcast conversion from vec2 f16.
 */
function bitcastVec2F16ToU32Comparator(vec2F16InU16x2) {
  assert(vec2F16InU16x2.length === 2);
  const expectations = possible32BitScalarIntervalsFromF16x2(vec2F16InU16x2, 'u32');
  // Return alwaysPass if result is expected unbounded.
  if (expectations.isUnbounded) {
    return anyU32;
  }
  return anyOf(...expectations.possibleExpectations);
}

/**
 * @returns a Comparator for checking if a i32 value is a valid
 * bitcast conversion from vec2 f16.
 */
function bitcastVec2F16ToI32Comparator(vec2F16InU16x2) {
  assert(vec2F16InU16x2.length === 2);
  const expectations = possible32BitScalarIntervalsFromF16x2(vec2F16InU16x2, 'i32');
  // Return alwaysPass if result is expected unbounded.
  if (expectations.isUnbounded) {
    return anyI32;
  }
  return anyOf(...expectations.possibleExpectations);
}

/**
 * @returns a Comparator for checking if a i32 value is a valid
 * bitcast conversion from vec2 f16.
 */
function bitcastVec2F16ToF32Comparator(vec2F16InU16x2) {
  assert(vec2F16InU16x2.length === 2);
  const expectations = possible32BitScalarIntervalsFromF16x2(vec2F16InU16x2, 'f32');
  // Return alwaysPass if result is expected unbounded.
  if (expectations.isUnbounded) {
    return anyF32;
  }
  return anyOf(...expectations.possibleExpectations);
}

/**
 * @returns a Comparator for checking if a vec2 u32 value is a valid
 * bitcast conversion from vec4 f16.
 */
function bitcastVec4F16ToVec2U32Comparator(vec4F16InU16x4) {
  assert(vec4F16InU16x4.length === 4);
  const expectationsPerElement = [vec4F16InU16x4.slice(0, 2), vec4F16InU16x4.slice(2, 4)].map((e) =>
  possible32BitScalarIntervalsFromF16x2(e, 'u32')
  );
  // Return alwaysPass if any element is expected unbounded. Although it may be only one unbounded
  // element in the result vector, currently we don't have a way to build a comparator that expect
  // only one element of i32/u32 vector unbounded.
  if (expectationsPerElement.map((e) => e.isUnbounded).reduce((a, b) => a || b, false)) {
    return alwaysPass('any vec2<u32>');
  }
  return anyOf(
    ...cartesianProduct(...expectationsPerElement.map((e) => e.possibleExpectations)).map(
      (e) => new VectorValue(e)
    )
  );
}

/**
 * @returns a Comparator for checking if a vec2 i32 value is a valid
 * bitcast conversion from vec4 f16.
 */
function bitcastVec4F16ToVec2I32Comparator(vec4F16InU16x4) {
  assert(vec4F16InU16x4.length === 4);
  const expectationsPerElement = [vec4F16InU16x4.slice(0, 2), vec4F16InU16x4.slice(2, 4)].map((e) =>
  possible32BitScalarIntervalsFromF16x2(e, 'i32')
  );
  // Return alwaysPass if any element is expected unbounded. Although it may be only one unbounded
  // element in the result vector, currently we don't have a way to build a comparator that expect
  // only one element of i32/u32 vector unbounded.
  if (expectationsPerElement.map((e) => e.isUnbounded).reduce((a, b) => a || b, false)) {
    return alwaysPass('any vec2<i32>');
  }
  return anyOf(
    ...cartesianProduct(...expectationsPerElement.map((e) => e.possibleExpectations)).map(
      (e) => new VectorValue(e)
    )
  );
}

/**
 * @returns a Comparator for checking if a vec2 f32 value is a valid
 * bitcast conversion from vec4 f16.
 */
function bitcastVec4F16ToVec2F32Comparator(vec4F16InU16x4) {
  assert(vec4F16InU16x4.length === 4);
  const expectationsPerElement = [vec4F16InU16x4.slice(0, 2), vec4F16InU16x4.slice(2, 4)].map((e) =>
  possible32BitScalarIntervalsFromF16x2(e, 'f32')
  );
  return anyOf(
    ...cartesianProduct(...expectationsPerElement.map((e) => e.possibleExpectations)).map((e) => [
    e[0],
    e[1]]
    )
  );
}

export const d = makeCaseCache('bitcast', {
  // Identity Cases
  i32_to_i32: () => fullI32Range().map((e) => ({ input: i32(e), expected: i32(e) })),
  u32_to_u32: () => fullU32Range().map((e) => ({ input: u32(e), expected: u32(e) })),
  f32_inf_nan_to_f32: () =>
  f32RangeWithInfAndNaN.map((e) => ({
    input: f32(e),
    expected: bitcastF32ToF32Comparator(e)
  })),
  f32_to_f32: () =>
  f32FiniteRange.map((e) => ({ input: f32(e), expected: bitcastF32ToF32Comparator(e) })),
  f16_inf_nan_to_f16: () =>
  [...f16FiniteInF16, ...f16InfAndNaNInF16].map((e) => ({
    input: f16(e),
    expected: bitcastF16ToF16Comparator(e)
  })),
  f16_to_f16: () =>
  f16FiniteInF16.map((e) => ({ input: f16(e), expected: bitcastF16ToF16Comparator(e) })),

  // i32,u32,f32,Abstract to different i32,u32,f32
  i32_to_u32: () => fullI32Range().map((e) => ({ input: i32(e), expected: u32(e) })),
  i32_to_f32: () =>
  i32RangeForF32Finite.map((e) => ({
    input: i32(e),
    expected: bitcastI32ToF32Comparator(e)
  })),
  ai_to_i32: () => fullI32Range().map((e) => ({ input: abstractInt(BigInt(e)), expected: i32(e) })),
  ai_to_u32: () => fullU32Range().map((e) => ({ input: abstractInt(BigInt(e)), expected: u32(e) })),
  ai_to_f32: () =>
  // AbstractInt is converted to i32, because there is no explicit overload
  i32RangeForF32Finite.map((e) => ({
    input: abstractInt(BigInt(e)),
    expected: bitcastI32ToF32Comparator(e)
  })),
  i32_to_f32_inf_nan: () =>
  i32RangeForF32FiniteInfNaN.map((e) => ({
    input: i32(e),
    expected: bitcastI32ToF32Comparator(e)
  })),
  u32_to_i32: () => fullU32Range().map((e) => ({ input: u32(e), expected: i32(e) })),
  u32_to_f32: () =>
  u32RangeForF32Finite.map((e) => ({
    input: u32(e),
    expected: bitcastU32ToF32Comparator(e)
  })),
  u32_to_f32_inf_nan: () =>
  u32RangeForF32FiniteInfNaN.map((e) => ({
    input: u32(e),
    expected: bitcastU32ToF32Comparator(e)
  })),
  f32_inf_nan_to_i32: () =>
  f32RangeWithInfAndNaN.map((e) => ({
    input: f32(e),
    expected: bitcastF32ToI32Comparator(e)
  })),
  f32_to_i32: () =>
  f32FiniteRange.map((e) => ({ input: f32(e), expected: bitcastF32ToI32Comparator(e) })),

  f32_inf_nan_to_u32: () =>
  f32RangeWithInfAndNaN.map((e) => ({
    input: f32(e),
    expected: bitcastF32ToU32Comparator(e)
  })),
  f32_to_u32: () =>
  f32FiniteRange.map((e) => ({ input: f32(e), expected: bitcastF32ToU32Comparator(e) })),

  // i32,u32,f32,AbstractInt to vec2<f16>
  u32_to_vec2_f16_inf_nan: () =>
  u32RangeForF16Vec2FiniteInfNaN.map((e) => ({
    input: u32(e),
    expected: bitcastU32ToVec2F16Comparator(e)
  })),
  u32_to_vec2_f16: () =>
  u32RangeForF16Vec2Finite.map((e) => ({
    input: u32(e),
    expected: bitcastU32ToVec2F16Comparator(e)
  })),
  i32_to_vec2_f16_inf_nan: () =>
  i32RangeForF16Vec2FiniteInfNaN.map((e) => ({
    input: i32(e),
    expected: bitcastI32ToVec2F16Comparator(e)
  })),
  i32_to_vec2_f16: () =>
  i32RangeForF16Vec2Finite.map((e) => ({
    input: i32(e),
    expected: bitcastI32ToVec2F16Comparator(e)
  })),
  ai_to_vec2_f16: () =>
  // AbstractInt is converted to i32, because there is no explicit overload
  i32RangeForF16Vec2Finite.map((e) => ({
    input: abstractInt(BigInt(e)),
    expected: bitcastI32ToVec2F16Comparator(e)
  })),
  f32_inf_nan_to_vec2_f16_inf_nan: () =>
  f32RangeWithInfAndNaNForF16Vec2FiniteInfNaN.map((e) => ({
    input: f32(e),
    expected: bitcastF32ToVec2F16Comparator(e)
  })),
  f32_to_vec2_f16: () =>
  f32FiniteRangeForF16Vec2Finite.map((e) => ({
    input: f32(e),
    expected: bitcastF32ToVec2F16Comparator(e)
  })),
  af_to_vec2_f16: () =>
  f32FiniteRangeForF16Vec2Finite.map((e) => ({
    input: abstractFloat(e),
    expected: bitcastF32ToVec2F16Comparator(e)
  })),

  // vec2<i32>, vec2<u32>, vec2<f32>, vec2<AbstractInt> to vec4<f16>
  vec2_i32_to_vec4_f16_inf_nan: () =>
  slidingSlice(i32RangeForF16Vec2FiniteInfNaN, 2).map((e) => ({
    input: toVector(e, i32),
    expected: bitcastVec2I32ToVec4F16Comparator(e)
  })),
  vec2_i32_to_vec4_f16: () =>
  slidingSlice(i32RangeForF16Vec2Finite, 2).map((e) => ({
    input: toVector(e, i32),
    expected: bitcastVec2I32ToVec4F16Comparator(e)
  })),
  vec2_ai_to_vec4_f16: () =>
  // AbstractInt is converted to i32, because there is no explicit overload
  slidingSlice(i32RangeForF16Vec2Finite, 2).map((e) => ({
    input: toVector(e, (n) => abstractInt(BigInt(n))),
    expected: bitcastVec2I32ToVec4F16Comparator(e)
  })),
  vec2_u32_to_vec4_f16_inf_nan: () =>
  slidingSlice(u32RangeForF16Vec2FiniteInfNaN, 2).map((e) => ({
    input: toVector(e, u32),
    expected: bitcastVec2U32ToVec4F16Comparator(e)
  })),
  vec2_u32_to_vec4_f16: () =>
  slidingSlice(u32RangeForF16Vec2Finite, 2).map((e) => ({
    input: toVector(e, u32),
    expected: bitcastVec2U32ToVec4F16Comparator(e)
  })),
  vec2_f32_inf_nan_to_vec4_f16_inf_nan: () =>
  slidingSlice(f32RangeWithInfAndNaNForF16Vec2FiniteInfNaN, 2).map((e) => ({
    input: toVector(e, f32),
    expected: bitcastVec2F32ToVec4F16Comparator(e)
  })),
  vec2_f32_to_vec4_f16: () =>
  slidingSlice(f32FiniteRangeForF16Vec2Finite, 2).map((e) => ({
    input: toVector(e, f32),
    expected: bitcastVec2F32ToVec4F16Comparator(e)
  })),
  vec2_af_to_vec4_f16: () =>
  slidingSlice(f32FiniteRangeForF16Vec2Finite, 2).map((e) => ({
    input: toVector(e, abstractFloat),
    expected: bitcastVec2F32ToVec4F16Comparator(e)
  })),

  // vec2<f16> to i32, u32, f32
  vec2_f16_to_u32: () =>
  f16Vec2FiniteInU16x2.map((e) => ({
    input: u16x2ToVec2F16(e),
    expected: bitcastVec2F16ToU32Comparator(e)
  })),
  vec2_f16_inf_nan_to_u32: () =>
  f16Vec2FiniteInfNanInU16x2.map((e) => ({
    input: u16x2ToVec2F16(e),
    expected: bitcastVec2F16ToU32Comparator(e)
  })),
  vec2_f16_to_i32: () =>
  f16Vec2FiniteInU16x2.map((e) => ({
    input: u16x2ToVec2F16(e),
    expected: bitcastVec2F16ToI32Comparator(e)
  })),
  vec2_f16_inf_nan_to_i32: () =>
  f16Vec2FiniteInfNanInU16x2.map((e) => ({
    input: u16x2ToVec2F16(e),
    expected: bitcastVec2F16ToI32Comparator(e)
  })),
  vec2_f16_to_f32_finite: () =>
  f16Vec2FiniteInU16x2.
  filter((u16x2) => isFiniteF32(reinterpretU32AsF32(u16x2ToU32(u16x2)))).
  map((e) => ({
    input: u16x2ToVec2F16(e),
    expected: bitcastVec2F16ToF32Comparator(e)
  })),
  vec2_f16_inf_nan_to_f32: () =>
  f16Vec2FiniteInfNanInU16x2.map((e) => ({
    input: u16x2ToVec2F16(e),
    expected: bitcastVec2F16ToF32Comparator(e)
  })),

  // vec4<f16> to vec2 of i32, u32, f32
  vec4_f16_to_vec2_u32: () =>
  f16Vec2FiniteInU16x4.map((e) => ({
    input: u16x4ToVec4F16(e),
    expected: bitcastVec4F16ToVec2U32Comparator(e)
  })),
  vec4_f16_inf_nan_to_vec2_u32: () =>
  f16Vec2FiniteInfNanInU16x4.map((e) => ({
    input: u16x4ToVec4F16(e),
    expected: bitcastVec4F16ToVec2U32Comparator(e)
  })),
  vec4_f16_to_vec2_i32: () =>
  f16Vec2FiniteInU16x4.map((e) => ({
    input: u16x4ToVec4F16(e),
    expected: bitcastVec4F16ToVec2I32Comparator(e)
  })),
  vec4_f16_inf_nan_to_vec2_i32: () =>
  f16Vec2FiniteInfNanInU16x4.map((e) => ({
    input: u16x4ToVec4F16(e),
    expected: bitcastVec4F16ToVec2I32Comparator(e)
  })),
  vec4_f16_to_vec2_f32_finite: () =>
  f16Vec2FiniteInU16x4.
  filter(
    (u16x4) =>
    isFiniteF32(reinterpretU32AsF32(u16x2ToU32(u16x4.slice(0, 2)))) &&
    isFiniteF32(reinterpretU32AsF32(u16x2ToU32(u16x4.slice(2, 4))))
  ).
  map((e) => ({
    input: u16x4ToVec4F16(e),
    expected: bitcastVec4F16ToVec2F32Comparator(e)
  })),
  vec4_f16_inf_nan_to_vec2_f32: () =>
  f16Vec2FiniteInfNanInU16x4.map((e) => ({
    input: u16x4ToVec4F16(e),
    expected: bitcastVec4F16ToVec2F32Comparator(e)
  }))
});