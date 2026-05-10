// Copyright (C) 2022 Apple Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.pop
description: >
  A TypeError is thrown when "length" is [[Set]] on an empty array with non-writable "length".
info: |
  Array.prototype.pop ( )

  [...]
  3. If len = 0, then
    a. Perform ? Set(O, "length", +0𝔽, true).

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
Object.defineProperty(array, "length", { writable: false });

assert.throws(TypeError, function() {
  array.pop();
});
