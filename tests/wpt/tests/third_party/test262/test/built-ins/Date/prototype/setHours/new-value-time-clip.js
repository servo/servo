// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.sethours
description: Behavior when new value exceeds [[DateValue]] limits
info: |
  1. Let t be LocalTime(? thisTimeValue(this value)).
  2. Let h be ? ToNumber(hour).
  3. If min is not specified, let m be MinFromTime(t); otherwise, let m be ?
     ToNumber(min).
  4. If sec is not specified, let s be SecFromTime(t); otherwise, let s be ?
     ToNumber(sec).
  5. If ms is not specified, let milli be msFromTime(t); otherwise, let milli
     be ? ToNumber(ms).
  6. Let date be MakeDate(Day(t), MakeTime(h, m, s, milli)).
  7. Let u be TimeClip(UTC(date)).
  8. Set the [[DateValue]] internal slot of this Date object to u.
  9. Return u.

  TimeClip (time)

  1. If time is not finite, return NaN.
  2. If abs(time) > 8.64 × 1015, return NaN.
---*/

var maxMs = 8.64e15;
var date = new Date(maxMs);
var returnValue;

assert.notSameValue(date.getTime(), NaN);

returnValue = date.setHours(24);

assert.sameValue(returnValue, NaN, 'overflow due to hours');

date = new Date(maxMs);

returnValue = date.setHours(0, 24 * 60);

assert.sameValue(returnValue, NaN, 'overflow due to minutes');

date = new Date(maxMs);

returnValue = date.setHours(0, 0, 24 * 60 * 60);

assert.sameValue(returnValue, NaN, 'overflow due to seconds');

date = new Date(maxMs);

returnValue = date.setHours(0, 0, 0, 24 * 60 * 60 * 1000);

assert.sameValue(returnValue, NaN, 'overflow due to milliseconds');
