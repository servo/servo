// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.setfullyear
description: Abrupt completion during type coercion of provided "year"
info: |
  1. Let t be LocalTime(? thisTimeValue(this value)).
  2. If t is NaN, let t be +0; otherwise, let t be LocalTime(t).
  3. Let y be ? ToNumber(year).
---*/

var date = new Date(0);
var callCount = 0;
var originalValue = date.getTime();
var obj = {
  valueOf: function() {
    throw new Test262Error();
  }
};
var counter = {
  valueOf: function() {
    callCount += 1;
  }
};

assert.throws(Test262Error, function() {
  date.setFullYear(obj, counter, counter);
});

assert.sameValue(date.getTime(), originalValue);
assert.sameValue(callCount, 0);
