// Copyright (C) 2016 The V8 Project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-isfinite-number
description: >
  Return abrupt completion from ToNumber(number)
info: |
  isFinite (number)

  1. Let num be ? ToNumber(number).
---*/

var obj1 = {
  valueOf: function() {
    throw new Test262Error();
  }
};

var obj2 = {
  toString: function() {
    throw new Test262Error();
  }
};

assert.throws(Test262Error, function() {
  isFinite(obj1);
}, "valueOf");

assert.throws(Test262Error, function() {
  isFinite(obj2);
}, "toString");
