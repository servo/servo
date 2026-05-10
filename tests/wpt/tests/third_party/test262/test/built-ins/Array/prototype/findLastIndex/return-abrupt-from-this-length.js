// Copyright (C) 2021 Microsoft. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.findlastindex
description: >
  Return abrupt from ToLength(Get(O, "length")).
info: |
  Array.prototype.findLastIndex ( predicate[ , thisArg ] )

  1. Let O be ? ToObject(this value).
  2. Let len be ? LengthOfArrayLike(O).
features: [array-find-from-last]
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
  [].findLastIndex.call(o1, function() {});
});

var o2 = {
  length: {
    valueOf: function() {
      throw new Test262Error();
    }
  }
};
assert.throws(Test262Error, function() {
  [].findLastIndex.call(o2, function() {});
});
