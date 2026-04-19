// Copyright (C) 2021 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.sethours
description: Order of coercion of provided arguments vs NaN check
info: |
  1. Let t be ? thisTimeValue(this value).
  2. Let h be ? ToNumber(hour).
  3. If min is present, let m be ? ToNumber(min).
  4. If sec is present, let s be ? ToNumber(sec).
  5. If ms is present, let milli be ? ToNumber(ms).
  6. If t is NaN, return NaN.
  7. Set t to LocalTime(t).
  8. If min is not present, let m be MinFromTime(t).
  9. If sec is not present, let s be SecFromTime(t).
  10. If ms is not present, let milli be msFromTime(t).
  11. Let date be MakeDate(Day(t), MakeTime(h, m, s, milli)).
  12. Let u be TimeClip(UTC(date)).
  13. Set the [[DateValue]] internal slot of this Date object to u.
  14. Return u.
includes: [compareArray.js]
---*/

var date = new Date(NaN);
var effects = [];
var argHour = {
  valueOf: function() {
    effects.push('valueOf hour');
    return 0;
  }
};
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

var returnValue = date.setHours(argHour, argMin, argSec, argMs);

var expectedEffects = ['valueOf hour', 'valueOf min', 'valueOf sec', 'valueOf ms'];

assert.compareArray(effects, expectedEffects);
assert.sameValue(returnValue, NaN, 'argument is ignored when `this` is an invalid date');
assert.sameValue(date.getTime(), NaN, 'argument is ignored when `this` is an invalid date');
