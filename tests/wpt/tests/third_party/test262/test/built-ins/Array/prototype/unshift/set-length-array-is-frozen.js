// Copyright (C) 2022 Apple Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.unshift
description: >
  A TypeError is thrown when "length" is [[Set]] on a frozen array.
info: |
  Array.prototype.unshift ( ...items )

  [...]
  4. If argCount > 0, then
    [...]
    d. Let j be +0𝔽.
    e. For each element E of items, do
        i. Perform ? Set(O, ! ToString(j), E, true).
        ii. Set j to j + 1𝔽.
  5. Perform ? Set(O, "length", 𝔽(len + argCount), true).

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
    Object.freeze(array);
    arrayPrototypeSet0Calls++;
  },
});

assert.throws(TypeError, function() {
  array.unshift(1);
});

assert(!array.hasOwnProperty(0));
assert.sameValue(array.length, 0);
assert.sameValue(arrayPrototypeSet0Calls, 1);
