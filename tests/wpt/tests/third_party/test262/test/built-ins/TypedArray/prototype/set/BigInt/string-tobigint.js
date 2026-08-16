// Copyright (C) 2018 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.set-array-offset
description: >
  Behavior for input array of Strings, successful conversion
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
  var typedArray = new TA(makeCtorArg(2));
  typedArray.set(['', '1'])

  assert.sameValue(typedArray[0], 0n);
  assert.sameValue(typedArray[1], 1n);

  assert.throws(SyntaxError, function() {
    typedArray.set(['1n']);
  }, "A StringNumericLiteral may not include a BigIntLiteralSuffix.");

  assert.throws(SyntaxError, function() {
    typedArray.set(["Infinity"]);
  }, "Replace the StrUnsignedDecimalLiteral production with DecimalDigits to not allow Infinity..");

  assert.throws(SyntaxError, function() {
    typedArray.set(["1.1"]);
  }, "Replace the StrUnsignedDecimalLiteral production with DecimalDigits to not allow... decimal points...");

  assert.throws(SyntaxError, function() {
    typedArray.set(["1e7"]);
  }, "Replace the StrUnsignedDecimalLiteral production with DecimalDigits to not allow... exponents...");

}, null, null, ["immutable"]);
