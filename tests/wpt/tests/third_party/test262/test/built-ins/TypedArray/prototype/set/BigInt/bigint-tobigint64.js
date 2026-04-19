// Copyright (C) 2018 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.set-array-offset
description: >
  Behavior for input array of BigInts
info: |
  %TypedArray%.prototype.set ( array [ , offset ] )
  Sets multiple values in this TypedArray, reading the values from the object
  array. The optional offset value indicates the first element index in this
  TypedArray where values are written. If omitted, it is assumed to be 0.
  ...
  21. Repeat, while targetByteIndex < limit
    a. Let Pk be ! ToString(k).
    b. Let kNumber be ? ToNumber(? Get(src, Pk)).
    c. Let value be ? Get(src, Pk).
    d. If target.[[TypedArrayName]] is "BigUint64Array" or "BigInt64Array",
       let value be ? ToBigInt(value).
    e. Otherwise, let value be ? ToNumber(value).
    f. If IsDetachedBuffer(targetBuffer) is true, throw a TypeError exception.
    g. Perform SetValueInBuffer(targetBuffer, targetByteIndex, targetType,
       kNumbervalue, true, "Unordered").
    h. Set k to k + 1.
    i. Set targetByteIndex to targetByteIndex + targetElementSize.
  ...

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
  Element Type: BigInt64
  Conversion Operation: ToBigInt64

  ToBigInt64 ( argument )
  The abstract operation ToBigInt64 converts argument to one of 264 integer
  values in the range -2^63 through 2^63-1, inclusive.
  This abstract operation functions as follows:
    1. Let n be ? ToBigInt(argument).
    2. Let int64bit be n modulo 2^64.
    3. If int64bit ≥ 2^63, return int64bit - 2^64; otherwise return int64bit.

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

var typedArray = new BigInt64Array(vals.length);
typedArray.set(vals);

assert.sameValue(typedArray[0], 2n,
                 "ToBigInt64(2n ** 64n + 2n) => 2n");

assert.sameValue(typedArray[1], -9223372036854775806n, // 2n - 2n ** 63n
                 "ToBigInt64(2n ** 63n + 2n) => -9223372036854775806n");

assert.sameValue(typedArray[2], 2n,
                 "ToBigInt64(2n) => 2n");

assert.sameValue(typedArray[3], 0n,
                 "ToBigInt64(0n) => 0n");

assert.sameValue(typedArray[4], -2n,
                 "ToBigInt64( -2n) => -2n");

assert.sameValue(typedArray[5], 9223372036854775806n, // 2n ** 63n - 2
                 "ToBigInt64( -(2n ** 64n) - 2n) => 9223372036854775806n");

assert.sameValue(typedArray[6], -2n,
                 "ToBigInt64( -(2n ** 64n) - 2n) => -2n");

