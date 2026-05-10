// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.slice
description: >
  TypeError is thrown if CreateDataProperty fails.
  (result object is non-extensible)
info: |
  Array.prototype.slice ( start, end )

  [...]
  10. Repeat, while k < final
    [...]
    c. If kPresent is true, then
      [...]
      ii. Perform ? CreateDataPropertyOrThrow(A, ! ToString(n), kValue).
    [...]

  CreateDataPropertyOrThrow ( O, P, V )

  [...]
  3. Let success be ? CreateDataProperty(O, P, V).
  4. If success is false, throw a TypeError exception.
features: [Symbol.species]
---*/

var A = function(_length) {
  this.length = 0;
  Object.preventExtensions(this);
};

var arr = [1];
arr.constructor = {};
arr.constructor[Symbol.species] = A;

assert.throws(TypeError, function() {
  arr.slice(0, 1);
});
