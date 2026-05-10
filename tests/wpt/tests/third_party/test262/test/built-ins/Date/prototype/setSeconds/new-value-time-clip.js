// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.setseconds
description: Behavior when new value exceeds [[DateValue]] limits
info: |
  1. Let t be LocalTime(? thisTimeValue(this value)).
  2. Let s be ? ToNumber(sec).
  3. If ms is not specified, let milli be msFromTime(t); otherwise, let milli
     be ? ToNumber(ms).
  4. Let date be MakeDate(Day(t), MakeTime(HourFromTime(t), MinFromTime(t), s,
     milli)).
  5. Let u be TimeClip(UTC(date)).
  6. Set the [[DateValue]] internal slot of this Date object to u.
  7. Return u.

  TimeClip (time)

  1. If time is not finite, return NaN.
  2. If abs(time) > 8.64 × 1015, return NaN.
---*/

var maxMs = 8.64e15;
var date = new Date(maxMs);
var returnValue;

assert.notSameValue(date.getTime(), NaN);

returnValue = date.setSeconds(24 * 60 * 60);

assert.sameValue(returnValue, NaN, 'overflow due to seconds');

date = new Date(maxMs);

returnValue = date.setSeconds(0, 24 * 60 * 60 * 1000);

assert.sameValue(returnValue, NaN, 'overflow due to milliseconds');
