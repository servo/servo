// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.slice
description: Abrupt completion from `@@species` property access
info: |
    [...]
    8. Let A be ? ArraySpeciesCreate(O, count).
    [...]

    9.4.2.3 ArraySpeciesCreate

    [...]
    5. Let C be ? Get(originalArray, "constructor").
    [...]
    7. If Type(C) is Object, then
       a. Let C be ? Get(C, @@species).
features: [Symbol.species]
---*/

var a = [];
a.constructor = {};

Object.defineProperty(a.constructor, Symbol.species, {
  get: function() {
    throw new Test262Error();
  }
});

assert.throws(Test262Error, function() {
  a.slice();
});
