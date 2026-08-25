// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.setmonth
description: Behavior when new value exceeds [[DateValue]] limits
info: |
  1. Let t be ? thisTimeValue(this value).
  2. If t is NaN, let t be +0; otherwise, let t be LocalTime(t).
  3. Let y be ? ToNumber(year).
  4. If month is not specified, let m be MonthFromTime(t); otherwise, let m be
     ? ToNumber(month).
  5. If date is not specified, let dt be DateFromTime(t); otherwise, let dt be
     ? ToNumber(date).
  6. Let newDate be MakeDate(MakeDay(y, m, dt), TimeWithinDay(t)).
  7. Let u be TimeClip(UTC(newDate)).
  8. Set the [[DateValue]] internal slot of this Date object to u.
  9. Return u.

  TimeClip (time)

  1. If time is not finite, return NaN.
  2. If abs(time) > 8.64 × 1015, return NaN.
---*/

var maxMs = 8.64e15;
var maxDate = 12;
var maxMonth = 8;
var date = new Date(maxMs);
var returnValue;

assert.notSameValue(date.getTime(), NaN);

returnValue = date.setMonth(maxMonth + 1);

assert.sameValue(returnValue, NaN, 'overflow due to month');

date = new Date(maxMs);

returnValue = date.setMonth(maxMonth, maxDate + 2);

assert.sameValue(returnValue, NaN, 'overflow due to date');
