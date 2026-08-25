// Copyright (C) 2021 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.setminutes
description: Order of coercion of provided argument vs NaN check
info: |
  1. Let t be ? thisTimeValue(this value).
  2. Let m be ? ToNumber(min).
  3. If sec is present, let s be ? ToNumber(sec).
  4. If ms is present, let milli be ? ToNumber(ms).
  5. If t is NaN, return NaN.
  6. Set t to LocalTime(t).
  7. If sec is not present, let s be SecFromTime(t).
  8. If ms is not present, let milli be msFromTime(t).
  9. Let date be MakeDate(Day(t), MakeTime(HourFromTime(t), m, s, milli)).
  10. Let u be TimeClip(UTC(date)).
  11. Set the [[DateValue]] internal slot of this Date object to u.
  12. Return u.
includes: [compareArray.js]
---*/

var date = new Date(NaN);
var effects = [];
var argMin = {
  valueOf: function() {
    effects.push('valueOf min');
    return 0;
  }
};
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

var returnValue = date.setMinutes(argMin, argSec, argMs);

var expectedEffects = ['valueOf min', 'valueOf sec', 'valueOf ms'];

assert.compareArray(effects, expectedEffects);
assert.sameValue(returnValue, NaN, 'argument is ignored when `this` is an invalid date');
assert.sameValue(date.getTime(), NaN, 'argument is ignored when `this` is an invalid date');
