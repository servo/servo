// Copyright (C) 2021 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.setutcseconds
description: Order of coercion of provided arguments vs NaN check
info: |
  1. Let t be ? thisTimeValue(this value).
  2. Let s be ? ToNumber(sec).
  3. If ms is present, let milli be ? ToNumber(ms).
  4. If t is NaN, return NaN.
  5. If ms is not present, let milli be msFromTime(t).
  6. Let date be MakeDate(Day(t), MakeTime(HourFromTime(t), MinFromTime(t), s, milli)).
  7. Let v be TimeClip(date).
  8. Set the [[DateValue]] internal slot of this Date object to v.
  9. Return v.
includes: [compareArray.js]
---*/

var date = new Date(NaN);
var effects = [];
var argSec = {
  valueOf: function() {
    effects.push('valueOf sec');
    return 0;
  }
};
var argMs = {
  valueOf: function() {
    effects.push('valueOf ms');
    return 0;
  }
};

var returnValue = date.setUTCSeconds(argSec, argMs);

var expectedEffects = ['valueOf sec', 'valueOf ms'];

assert.compareArray(effects, expectedEffects);
assert.sameValue(returnValue, NaN, 'argument is ignored when `this` is an invalid date');
assert.sameValue(date.getTime(), NaN, 'argument is ignored when `this` is an invalid date');
