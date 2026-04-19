// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-integer-indexed-exotic-objects-defineownproperty-p-desc
description: >
    Defining a typed array element to a value that, when converted to the typed
    array element type, detaches the typed array's underlying buffer, should
    return true and not modify the typed array.
info: |
  9.4.5.3 [[DefineOwnProperty]] ( P, Desc )

  ...
  3. If Type(P) is String, then
    a. Let numericIndex be ! CanonicalNumericIndexString(P).
    b. If numericIndex is not undefined, then
      ...
      x. If Desc has a [[Value]] field, then
        1. Let value be Desc.[[Value]].
        2. Return ? IntegerIndexedElementSet(O, numericIndex, value).
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

includes: [testTypedArray.js, detachArrayBuffer.js]
features: [align-detached-buffer-semantics-with-web-reality, Reflect, TypedArray]
---*/

testWithTypedArrayConstructors(function(TA) {
  var ta = new TA([17]);

  var desc =
    {
      value: {
        valueOf() {
          $DETACHBUFFER(ta.buffer);
          return 42;
        }
      }
    };

  assert.sameValue(
    Reflect.defineProperty(ta, 0, desc),
    true,
    'Reflect.defineProperty(ta, 0, {value: {valueOf() {$DETACHBUFFER(ta.buffer); return 42;}}} ) must return true'
  );
  assert.sameValue(ta[0], undefined, 'The value of ta[0] is expected to equal `undefined`');
}, null, ["passthrough"]);
