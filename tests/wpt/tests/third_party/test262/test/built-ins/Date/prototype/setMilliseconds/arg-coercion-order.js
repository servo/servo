// Copyright (C) 2021 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.setmilliseconds
description: Order of coercion of provided argument vs NaN check
info: |
  1. Let t be ? thisTimeValue(this value).
  2. Set ms to ? ToNumber(ms).
  3. If t is NaN, return NaN.
  4. Set t to LocalTime(t).
  5. Let time be MakeTime(HourFromTime(t), MinFromTime(t), SecFromTime(t), ms).
  6. Let u be TimeClip(UTC(MakeDate(Day(t), time))).
  7. Set the [[DateValue]] internal slot of this Date object to u.
  8. Return u.
---*/

var date = new Date(NaN);
var callCount = 0;
var arg = {
  valueOf: function() {
    callCount += 1;
    return 0;
  }
};

var returnValue = date.setMilliseconds(arg);

assert.sameValue(callCount, 1, 'ToNumber invoked exactly once');
assert.sameValue(returnValue, NaN, 'argument is ignored when `this` is an invalid date');
assert.sameValue(date.getTime(), NaN, 'argument is ignored when `this` is an invalid date');
