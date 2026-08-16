// Copyright (C) 2018 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.set-array-offset
description: >
  Return abrupt on Number
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
    Argument Type: Number
    Result: Throw a TypeError exception.

includes: [testTypedArray.js]
features: [BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var typedArray = new TA(makeCtorArg(1));

  assert.throws(TypeError, function() {
    typedArray.set([1]);
  }, "abrupt completion from Number: 1");

  assert.throws(TypeError, function() {
    typedArray.set([Math.pow(2, 63)]);
  }, "abrupt completion from Number: 2**63");

  assert.throws(TypeError, function() {
    typedArray.set([+0]);
  }, "abrupt completion from Number: +0");

  assert.throws(TypeError, function() {
    typedArray.set([-0]);
  }, "abrupt completion from Number: -0");

  assert.throws(TypeError, function() {
    typedArray.set([Infinity]);
  }, "abrupt completion from Number: Infinity");

  assert.throws(TypeError, function() {
    typedArray.set([-Infinity]);
  }, "abrupt completion from Number: -Infinity");

  assert.throws(TypeError, function() {
    typedArray.set([NaN]);
  }, "abrupt completion from Number: NaN");

}, null, null, ["immutable"]);
