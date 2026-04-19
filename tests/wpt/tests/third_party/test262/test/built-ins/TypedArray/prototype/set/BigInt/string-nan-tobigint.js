// Copyright (C) 2018 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.set-array-offset
description: >
  Return abrupt String, when StringToBigInt returns NaN
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

  ToBigInt ( argument )
  Object, Apply the following steps:
    1. Let prim be ? ToPrimitive(argument, hint Number).
    2. Return the value that prim corresponds to in Table [BigInt Conversions]

  BigInt Conversions
    Argument Type: String
    Result:
      1. Let n be StringToBigInt(prim).
      2. If n is NaN, throw a SyntaxError exception.
      3. Return n.

includes: [testTypedArray.js]
features: [BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var typedArray = new TA(makeCtorArg(1));

  assert.throws(SyntaxError, function() {
    typedArray.set(["definately not a number"]);
  }, "StringToBigInt(prim) == NaN");

});
