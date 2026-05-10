// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.findindex
description: >
  Return abrupt from ToLength(Get(O, "length")).
info: |
  22.1.3.9 Array.prototype.findIndex ( predicate[ , thisArg ] )

  1. Let O be ToObject(this value).
  2. ReturnIfAbrupt(O).
  3. Let len be ToLength(Get(O, "length")).
  4. ReturnIfAbrupt(len).
---*/

var o1 = {};

Object.defineProperty(o1, 'length', {
  get: function() {
    throw new Test262Error();
  },
  configurable: true
});
// predicate fn is given to avoid false positives
assert.throws(Test262Error, function() {
  [].findIndex.call(o1, function() {});
});

var o2 = {
  length: {
    valueOf: function() {
      throw new Test262Error();
    }
  }
};
assert.throws(Test262Error, function() {
  [].findIndex.call(o2, function() {});
});
