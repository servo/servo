// Copyright (C) 2021 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.setmonth
description: Order of coercion of provided argument vs NaN check
info: |
  1. Let t be ? thisTimeValue(this value).
  2. Let m be ? ToNumber(month).
  3. If date is present, let dt be ? ToNumber(date).
  4. If t is NaN, return NaN.
  5. Set t to LocalTime(t).
  6. If date is not present, let dt be DateFromTime(t).
  7. Let newDate be MakeDate(MakeDay(YearFromTime(t), m, dt), TimeWithinDay(t)).
  8. Let u be TimeClip(UTC(newDate)).
  9. Set the [[DateValue]] internal slot of this Date object to u.
  10. Return u.
includes: [compareArray.js]
---*/

var date = new Date(NaN);
var effects = [];
var argMonth = {
  valueOf: function() {
    effects.push('valueOf month');
    return 0;
  }
};
var argDate = {
  valueOf: function() {
    effects.push('valueOf date');
    return 0;
  }
};

var returnValue = date.setMonth(argMonth, argDate);

var expectedEffects = ['valueOf month', 'valueOf date'];

assert.compareArray(effects, expectedEffects);
assert.sameValue(returnValue, NaN, 'argument is ignored when `this` is an invalid date');
assert.sameValue(date.getTime(), NaN, 'argument is ignored when `this` is an invalid date');
