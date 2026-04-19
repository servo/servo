// Copyright (C) 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-date-year-month-date-hours-minutes-seconds-ms
description: Abrupt completions from coercing input values
info: |
  3. If NewTarget is not undefined, then
    a. Let y be ? ToNumber(year).
    b. Let m be ? ToNumber(month).
    c. If date is supplied, let dt be ? ToNumber(date); else let dt be 1.
    d. If hours is supplied, let h be ? ToNumber(hours); else let h be 0.
    e. If minutes is supplied, let min be ? ToNumber(minutes); else let min be 0.
    f. If seconds is supplied, let s be ? ToNumber(seconds); else let s be 0.
    g. If ms is supplied, let milli be ? ToNumber(ms); else let milli be 0.
    h. If y is not NaN and 0 ≤ ToInteger(y) ≤ 99, let yr be 1900+ToInteger(y); otherwise,
      let yr be y.
    i. Let finalDate be MakeDate(MakeDay(yr, m, dt), MakeTime(h, min, s, milli)).
    j. Let O be ? OrdinaryCreateFromConstructor(NewTarget, "%DatePrototype%", « [[DateValue]] »).
    k. Set O.[[DateValue]] to TimeClip(UTC(finalDate)).
    l. Return O.
---*/

var thrower = { toString: function() { throw new Test262Error(); } };
var counter = { toString: function() { callCount += 1; } };
var callCount = 0;

assert.throws(Test262Error, function() {
  new Date(thrower, counter);
}, 'year');
assert.sameValue(callCount, 0, 'coercion halts following error from "year"');

assert.throws(Test262Error, function() {
  new Date(0, thrower, counter);
}, 'month');
assert.sameValue(callCount, 0, 'coercion halts following error from "month"');

assert.throws(Test262Error, function() {
  new Date(0, 0, thrower, counter);
}, 'date');
assert.sameValue(callCount, 0, 'coercion halts following error from "date"');

assert.throws(Test262Error, function() {
  new Date(0, 0, 1, thrower, counter);
}, 'hours');
assert.sameValue(callCount, 0, 'coercion halts following error from "hours"');

assert.throws(Test262Error, function() {
  new Date(0, 0, 1, 0, thrower, counter);
}, 'minutes');
assert.sameValue(
  callCount, 0, 'coercion halts following error from "minutes"'
);

assert.throws(Test262Error, function() {
  new Date(0, 0, 1, 0, 0, thrower, counter);
}, 'seconds');
assert.sameValue(
  callCount, 0, 'coercion halts following error from "seconds"'
);

assert.throws(Test262Error, function() {
  new Date(0, 0, 1, 0, 0, 0, thrower);
}, 'ms');
