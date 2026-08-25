// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
description: >
  TypeError is thrown if CreateDataProperty fails.
  (result object's "0" is non-configurable)
info: |
  Array.prototype.filter ( callbackfn [ , thisArg ] )

  [...]
  7. Repeat, while k < len
    [...]
    c. If kPresent is true, then
      [...]
      iii. If selected is true, then
        1. Perform ? CreateDataPropertyOrThrow(A, ! ToString(to), kValue).
  [...]

  CreateDataPropertyOrThrow ( O, P, V )

  [...]
  3. Let success be ? CreateDataProperty(O, P, V).
  4. If success is false, throw a TypeError exception.
features: [Symbol.species]
---*/

var A = function(_length) {
  Object.defineProperty(this, "0", {
    writable: true,
    configurable: false,
  });
};

var arr = [1];
arr.constructor = {};
arr.constructor[Symbol.species] = A;

assert.throws(TypeError, function() {
  arr.filter(function() {
    return true;
  });
});
