// Copyright (C) 2018 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-typedarray-object
description: >
  Return abrupt on null
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

  ToBigInt ( argument )
  Object, Apply the following steps:
    1. Let prim be ? ToPrimitive(argument, hint Number).
    2. Return the value that prim corresponds to in Table [BigInt Conversions]

  BigInt Conversions
    Argument Type: Null
    Result: Throw a TypeError exception.

includes: [testTypedArray.js]
features: [BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {

  assert.throws(TypeError, function() {
    new TA(makeCtorArg([null]));
  }, "abrupt completion from Null");

});
