// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-set-p-v-receiver
description: >
  Returns true after setting value
info: |
  9.4.5.5 [[Set]] ( P, V, Receiver)

  ...
  2. If Type(P) is String, then
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

includes: [testTypedArray.js]
features: [align-detached-buffer-semantics-with-web-reality, Reflect, TypedArray]
---*/

let proto = TypedArray.prototype;
let throwDesc = {
  set: function() {
    throw new Test262Error('OrdinarySet was called!');
  }
};

Object.defineProperty(proto, '0', throwDesc);
Object.defineProperty(proto, '1', throwDesc);

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  let sample = new TA(makeCtorArg(2));
  assert.sameValue(Reflect.set(sample, '0', 1), true, 'Reflect.set(sample, "0", 1) must return true');
  assert.sameValue(sample[0], 1, 'The value of sample[0] is 1');
  assert.sameValue(Reflect.set(sample, '1', 42), true, 'Reflect.set(sample, "1", 42) must return true');
  assert.sameValue(sample[1], 42, 'The value of sample[1] is 42');
}, null, ["passthrough"]);
