// Copyright (C) 2018 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-typedarray-object
description: >
  Behavior for input array of BigInts
info: |
  TypedArray ( object )
  This description applies only if the TypedArray function is called with at
  least one argument and the Type of the first argument is Object and that
  object does not have either a [[TypedArrayName]] or an [[ArrayBufferData]]
  internal slot.
  ...
  8. Repeat, while k < len
    ...
    b. Let kValue be ? Get(arrayLike, Pk).
    c. Perform ? Set(O, Pk, kValue, true).
  ...

  [[Set]] ( P, V, Receiver)
  ...
  2. If Type(P) is String and if SameValue(O, Receiver) is true, then
    a. Let numericIndex be ! CanonicalNumericIndexString(P).
    b. If numericIndex is not undefined, then
      i. Perform ? IntegerIndexedElementSet(O, numericIndex, V).
      ii. Return true.
  ...

  IntegerIndexedElementSet ( O, index, value )

  Assert: O is an Integer-Indexed exotic object.
  If O.[[ContentType]] is BigInt, let numValue be ? ToBigInt(value).
  Otherwise, let numValue be ? ToNumber(value).
  Let buffer be O.[[ViewedArrayBuffer]].
  If IsDetachedBuffer(buffer) is false and ! IsValidIntegerIndex(O, index) is true, then
    Let offset be O.[[ByteOffset]].
    Let arrayTypeName be the String value of O.[[TypedArrayName]].
    Let elementSize be the Element Size value specified in Table 62 for arrayTypeName.
    Let indexedPosition be (ℝ(index) × elementSize) + offset.
    Let elementType be the Element Type value in Table 62 for arrayTypeName.
    Perform SetValueInBuffer(buffer, indexedPosition, elementType, numValue, true, Unordered).
  Return NormalCompletion(undefined).

  SetValueInBuffer ( arrayBuffer, byteIndex, type, value, isTypedArray, order [ , isLittleEndian ] )
  ...
  8. Let rawBytes be NumberToRawBytes(type, value, isLittleEndian).
  ...

  NumberToRawBytes( type, value, isLittleEndian )
  ...
  3. Else,
    a. Let n be the Number value of the Element Size specified in Table
       [The TypedArray Constructors] for Element Type type.
    b. Let convOp be the abstract operation named in the Conversion Operation
       column in Table 9 for Element Type type.

  The TypedArray Constructors
  Element Type: BigUint64
  Conversion Operation: ToBigUint64

  ToBigUint64 ( argument )
  The abstract operation ToBigInt64 converts argument to one of 264 integer
  values in the range -2^63 through 2^63-1, inclusive.
  This abstract operation functions as follows:
    1. Let n be ? ToBigInt(argument).
    2. Let int64bit be n modulo 2^64.
    3. Return int64bit.

features: [BigInt, TypedArray]
---*/

var vals = [
  18446744073709551618n, // 2n ** 64n + 2n
  9223372036854775810n, // 2n ** 63n + 2n
  2n,
  0n,
  -2n,
  -9223372036854775810n, // -(2n ** 63n) - 2n
  -18446744073709551618n, // -(2n ** 64n) - 2n
];

var typedArray = new BigUint64Array(vals);


assert.sameValue(typedArray[0], 2n,
                 "ToBigUint64(2n ** 64n + 2n) => 2n");

assert.sameValue(typedArray[1], 9223372036854775810n, // 2n ** 63n + 2n
                 "ToBigUint64(2n ** 63n + 2n) => 9223372036854775810");

assert.sameValue(typedArray[2], 2n,
                 "ToBigUint64(2n) => 2n");

assert.sameValue(typedArray[3], 0n,
                 "ToBigUint64(0n) => 0n");

assert.sameValue(typedArray[4], 18446744073709551614n, // 2n ** 64n - 2n
                 "ToBigUint64( -2n) => 18446744073709551614n");

assert.sameValue(typedArray[5], 9223372036854775806n, // 2n ** 63n - 2n
                 "ToBigUint64( -(2n ** 63n) - 2n) => 9223372036854775806n");

assert.sameValue(typedArray[6], 18446744073709551614n, // 2n ** 64n - 2n
                 "ToBigUint64( -(2n ** 64n) - 2n) => 18446744073709551614n");
