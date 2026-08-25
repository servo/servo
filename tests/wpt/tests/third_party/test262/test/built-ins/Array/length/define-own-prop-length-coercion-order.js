// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: André Bargull
esid: sec-arraysetlength
description: >
  [[Value]] is coerced to number before descriptor validation.
info: |
  ArraySetLength ( A, Desc )

  [...]
  3. Let newLen be ? ToUint32(Desc.[[Value]]).
  4. Let numberLen be ? ToNumber(Desc.[[Value]]).
  [...]
  7. Let oldLenDesc be OrdinaryGetOwnProperty(A, "length").
  [...]
  11. If newLen ≥ oldLen, then
    a. Return OrdinaryDefineOwnProperty(A, "length", newLenDesc).

  OrdinaryDefineOwnProperty ( O, P, Desc )

  [...]
  3. Return ValidateAndApplyPropertyDescriptor(O, P, extensible, Desc, current).

  ValidateAndApplyPropertyDescriptor ( O, P, extensible, Desc, current )

  [...]
  7. Else if IsDataDescriptor(current) and IsDataDescriptor(Desc) are both true, then
    a. If current.[[Configurable]] is false and current.[[Writable]] is false, then
      i. If Desc.[[Writable]] is present and Desc.[[Writable]] is true, return false.
features: [Reflect]
---*/

var array = [1, 2];
var valueOfCalls = 0;
var length = {
  valueOf: function() {
    valueOfCalls += 1;
    if (valueOfCalls !== 1) {
      // skip first coercion at step 3
      Object.defineProperty(array, "length", {writable: false});
    }
    return array.length;
  },
};

assert.throws(TypeError, function() {
  Object.defineProperty(array, "length", {value: length, writable: true});
}, 'Object.defineProperty(array, "length", {value: length, writable: true}) throws a TypeError exception');
assert.sameValue(valueOfCalls, 2, 'The value of valueOfCalls is expected to be 2');


array = [1, 2];
valueOfCalls = 0;

assert(
  !Reflect.defineProperty(array, "length", {value: length, writable: true}),
  'The value of !Reflect.defineProperty(array, "length", {value: length, writable: true}) is expected to be true'
);
assert.sameValue(valueOfCalls, 2, 'The value of valueOfCalls is expected to be 2');
