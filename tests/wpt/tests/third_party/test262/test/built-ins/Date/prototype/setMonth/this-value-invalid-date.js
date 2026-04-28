// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.setmonth
description: >
  Behavior when the "this" value is a Date object describing an invald date
info: |
  1. Let t be LocalTime(? thisTimeValue(this value)).
  2. Let m be ? ToNumber(month).
  3. If date is not specified, let dt be DateFromTime(t); otherwise, let dt be
     ? ToNumber(date).
  4. Let newDate be MakeDate(MakeDay(YearFromTime(t), m, dt),
     TimeWithinDay(t)).
  5. Let u be TimeClip(UTC(newDate)).
  6. Set the [[DateValue]] internal slot of this Date object to u.
  7. Return u.
---*/

var date = new Date(NaN);
var result;

result = date.setMonth(6);

assert.sameValue(result, NaN, 'return value (month)');
assert.sameValue(date.getTime(), NaN, '[[DateValue]] internal slot (month)');

date = new Date(NaN);

result = date.setMonth(6, 7);

assert.sameValue(result, NaN, 'return value (date)');
assert.sameValue(date.getTime(), NaN, '[[DateValue]] internal slot (month)');
