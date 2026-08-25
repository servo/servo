// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.settime
description: Behavior when new value exceeds [[DateValue]] limits
info: |
  1. Perform ? thisTimeValue(this value).
  2. Let t be ? ToNumber(time).
  3. Let v be TimeClip(t).
  4. Set the [[DateValue]] internal slot of this Date object to v.
  5. Return v.

  TimeClip (time)

  1. If time is not finite, return NaN.
  2. If abs(time) > 8.64 × 1015, return NaN.
---*/

var maxMs = 8.64e15;
var date = new Date(0);
var returnValue;

assert.notSameValue(date.getTime(), NaN);

returnValue = date.setTime(maxMs + 1);

assert.sameValue(returnValue, NaN);
