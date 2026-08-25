// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.filter
description: Abrupt completion from `constructor` property access
info: |
    [...]
    5. Let A be ? ArraySpeciesCreate(O, 0).
    [...]

    9.4.2.3 ArraySpeciesCreate

    [...]
    5. Let C be ? Get(originalArray, "constructor").
---*/

var a = [];
var callCount = 0;
var cb = function() {
  callCount += 1;
};

Object.defineProperty(a, 'constructor', {
  get: function() {
    throw new Test262Error();
  }
});

assert.throws(Test262Error, function() {
  a.filter(cb);
});
assert.sameValue(callCount, 0);
