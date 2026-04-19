// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.slice
description: Abrupt completion from `constructor` property access
info: |
    [...]
    8. Let A be ? ArraySpeciesCreate(O, count).
    [...]

    9.4.2.3 ArraySpeciesCreate

    [...]
    5. Let C be ? Get(originalArray, "constructor").
---*/

var a = [];

Object.defineProperty(a, 'constructor', {
  get: function() {
    throw new Test262Error();
  }
});

assert.throws(Test262Error, function() {
  a.slice();
});
