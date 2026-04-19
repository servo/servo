// Copyright (C) 2022 Apple Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.push
description: >
  A TypeError is thrown when "length" is [[Set]] on an array with non-writable "length".
info: |
  Array.prototype.push ( ...items )

  [...]
  5. For each element E of items, do
    a. Perform ? Set(O, ! ToString(𝔽(len)), E, true).
    b. Set len to len + 1.
  6. Perform ? Set(O, "length", 𝔽(len), true).

  OrdinarySetWithOwnDescriptor ( O, P, V, Receiver, ownDesc )

  [...]
  2. If IsDataDescriptor(ownDesc) is true, then
    a. If ownDesc.[[Writable]] is false, return false.

  Set ( O, P, V, Throw )

  [...]
  1. Let success be ? O.[[Set]](P, V, O).
  2. If success is false and Throw is true, throw a TypeError exception.
---*/

var array = [];
var arrayPrototypeSet0Calls = 0;

Object.defineProperty(Array.prototype, "0", {
  set(_val) {
    Object.defineProperty(array, "length", { writable: false });
    arrayPrototypeSet0Calls++;
  },
});

assert.throws(TypeError, function() {
  array.push(1);
});

assert(!array.hasOwnProperty(0));
assert.sameValue(array.length, 0);
assert.sameValue(arrayPrototypeSet0Calls, 1);
