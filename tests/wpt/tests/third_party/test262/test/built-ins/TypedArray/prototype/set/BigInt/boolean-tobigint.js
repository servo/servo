// Copyright (C) 2018 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.set-array-offset
description: >
  Behavior for input array of Booleans
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
    Argument Type: Boolean
    Result: Return 1n if prim is true and 0n if prim is false.

includes: [testTypedArray.js]
features: [BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var typedArray = new TA(makeCtorArg(2));
  typedArray.set([false, true])

  assert.sameValue(typedArray[0], 0n, "False converts to BigInt");
  assert.sameValue(typedArray[1], 1n, "True converts to BigInt");
});
