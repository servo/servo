// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.utc
description: Abrupt completions from coercing input values
info: |
  1. Let y be ? ToNumber(year).
  2. Let m be ? ToNumber(month).
  3. If date is supplied, let dt be ? ToNumber(date); else let dt be 1.
  4. If hours is supplied, let h be ? ToNumber(hours); else let h be 0.
  5. If minutes is supplied, let min be ? ToNumber(minutes); else let min be 0.
  6. If seconds is supplied, let s be ? ToNumber(seconds); else let s be 0.
  7. If ms is supplied, let milli be ? ToNumber(ms); else let milli be 0.
  8. If y is not NaN and 0 ≤ ToInteger(y) ≤ 99, let yr be 1900+ToInteger(y);
     otherwise, let yr be y.
  9. Return TimeClip(MakeDate(MakeDay(yr, m, dt), MakeTime(h, min, s, milli))). 
---*/

var thrower = { toString: function() { throw new Test262Error(); } };
var counter = { toString: function() { callCount += 1; } };
var callCount = 0;

assert.throws(Test262Error, function() {
  Date.UTC(thrower, counter);
}, 'year');
assert.sameValue(callCount, 0, 'coercion halts following error from "year"');

assert.throws(Test262Error, function() {
  Date.UTC(0, thrower, counter);
}, 'month');
assert.sameValue(callCount, 0, 'coercion halts following error from "month"');

assert.throws(Test262Error, function() {
  Date.UTC(0, 0, thrower, counter);
}, 'date');
assert.sameValue(callCount, 0, 'coercion halts following error from "date"');

assert.throws(Test262Error, function() {
  Date.UTC(0, 0, 1, thrower, counter);
}, 'hours');
assert.sameValue(callCount, 0, 'coercion halts following error from "hours"');

assert.throws(Test262Error, function() {
  Date.UTC(0, 0, 1, 0, thrower, counter);
}, 'minutes');
assert.sameValue(
  callCount, 0, 'coercion halts following error from "minutes"'
);

assert.throws(Test262Error, function() {
  Date.UTC(0, 0, 1, 0, 0, thrower, counter);
}, 'seconds');
assert.sameValue(
  callCount, 0, 'coercion halts following error from "seconds"'
);

assert.throws(Test262Error, function() {
  Date.UTC(0, 0, 1, 0, 0, 0, thrower);
}, 'ms');
