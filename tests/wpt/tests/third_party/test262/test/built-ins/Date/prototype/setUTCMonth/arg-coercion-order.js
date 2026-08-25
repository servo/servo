// Copyright (C) 2021 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.setutcmonth
description: Order of coercion of provided argument vs NaN check
info: |
  1. Let t be ? thisTimeValue(this value).
  2. Let m be ? ToNumber(month).
  3. If date is present, let dt be ? ToNumber(date).
  4. If t is NaN, return NaN.
  5. If date is not present, let dt be DateFromTime(t).
  6. Let newDate be MakeDate(MakeDay(YearFromTime(t), m, dt), TimeWithinDay(t)).
  7. Let v be TimeClip(newDate).
  8. Set the [[DateValue]] internal slot of this Date object to v.
  9. Return v.
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

var returnValue = date.setUTCMonth(argMonth, argDate);

var expectedEffects = ['valueOf month', 'valueOf date'];

assert.compareArray(effects, expectedEffects);
assert.sameValue(returnValue, NaN, 'argument is ignored when `this` is an invalid date');
assert.sameValue(date.getTime(), NaN, 'argument is ignored when `this` is an invalid date');
