// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-typedarray-object
description: >
  Return abrupt from valueOf() when setting a property
info: |
  22.2.4.4 TypedArray ( object )

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

  9.4.5.5 [[Set]] ( P, V, Receiver)

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

  ToNumber ( argument )

  Object, Apply the following steps:

    1. Let primValue be ? ToPrimitive(argument, hint Number).
    2. Return ? ToNumber(primValue).

  7.1.1 ToPrimitive ( input [ , PreferredType ] )

  ...
  4. Let exoticToPrim be ? GetMethod(input, @@toPrimitive).
  5. If exoticToPrim is not undefined, then
    a. Let result be ? Call(exoticToPrim, input, « hint »).
    b. If Type(result) is not Object, return result.
    c. Throw a TypeError exception.
  ...
  7. Return ? OrdinaryToPrimitive(input, hint).

  OrdinaryToPrimitive

  ...
  5. For each name in methodNames in List order, do
    a. Let method be ? Get(O, name).
    b. If IsCallable(method) is true, then
      i. Let result be ? Call(method, O).
      ii. If Type(result) is not Object, return result.
  6. Throw a TypeError exception.
includes: [testTypedArray.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new Int8Array(1);
  var valueOf = 0;

  sample.valueOf = function() {
    valueOf++;
    throw new Test262Error();
  };

  assert.throws(Test262Error, function() {
    new TA(makeCtorArg([8, sample]));
  }, "abrupt completion from ToNumber(sample)");

  assert.sameValue(valueOf, 1, "valueOf called once");
});
