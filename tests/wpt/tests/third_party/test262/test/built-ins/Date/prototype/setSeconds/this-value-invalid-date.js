// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.setseconds
description: >
  Behavior when the "this" value is a Date object describing an invald date
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
---*/

var date = new Date(NaN);
var result;

result = date.setSeconds(0);

assert.sameValue(result, NaN, 'return value (second)');
assert.sameValue(date.getTime(), NaN, '[[DateValue]] internal slot (second)');

result = date.setSeconds(0, 0);

assert.sameValue(result, NaN, 'return value (ms)');
assert.sameValue(date.getTime(), NaN, '[[DateValue]] internal slot (ms)');
