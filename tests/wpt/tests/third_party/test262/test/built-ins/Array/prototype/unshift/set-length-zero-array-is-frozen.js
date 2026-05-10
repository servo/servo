// Copyright (C) 2022 Apple Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.unshift
description: >
  A TypeError is thrown when "length" is [[Set]] on an empty frozen array.
info: |
  Array.prototype.unshift ( ...items )

  [...]
  2. Let len be ? LengthOfArrayLike(O).
  [...]
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
Object.freeze(array);

assert.throws(TypeError, function() {
  array.unshift();
});

assert(!array.hasOwnProperty(0));
assert.sameValue(array.length, 0);
