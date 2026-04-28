// Copyright (C) 2022 Apple Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.pop
description: >
  A TypeError is thrown when "length" is [[Set]] on an array with non-writable "length".
info: |
  Array.prototype.pop ( )

  [...]
  4. Else,
    a. Assert: len > 0.
    b. Let newLen be 𝔽(len - 1).
    c. Let index be ! ToString(newLen).
    d. Let element be ? Get(O, index).
    e. Perform ? DeletePropertyOrThrow(O, index).
    f. Perform ? Set(O, "length", newLen, true).

  OrdinarySetWithOwnDescriptor ( O, P, V, Receiver, ownDesc )

  [...]
  2. If IsDataDescriptor(ownDesc) is true, then
    a. If ownDesc.[[Writable]] is false, return false.

  Set ( O, P, V, Throw )

  [...]
  1. Let success be ? O.[[Set]](P, V, O).
  2. If success is false and Throw is true, throw a TypeError exception.
---*/

var array = new Array(1);
var arrayPrototypeGet0Calls = 0;

Object.defineProperty(Array.prototype, "0", {
  get() {
    Object.defineProperty(array, "length", { writable: false });
    arrayPrototypeGet0Calls++;
  },
});

assert.throws(TypeError, function() {
  array.pop();
});

assert.sameValue(array.length, 1);
assert.sameValue(arrayPrototypeGet0Calls, 1);
