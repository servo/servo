// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-number.prototype.toprecision
description: >
  Throws a RangeError if precision cannot be coerced to a number in range.
info: |
  Let p be ? ToInteger(precision).
  If x is not finite, return ! Number::toString(x).
  If p < 1 or p > 100, throw a RangeError exception.

features: [Symbol]
---*/

var toPrecision = Number.prototype.toPrecision;

assert.throws(RangeError, function() {
  toPrecision.call(1, function() {});
}, "`function() {}` doesn't coerce into a number in range (1-100)");

assert.throws(RangeError, function() {
  toPrecision.call(1, NaN);
}, "NaN doesn't coerce into a number in range (1-100)");

assert.throws(RangeError, function() {
  toPrecision.call(1, {});
}, "{} doesn't coerce into a number in range (1-100)");
