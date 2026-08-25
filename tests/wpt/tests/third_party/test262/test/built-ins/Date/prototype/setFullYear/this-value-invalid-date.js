// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.setfullyear
description: >
  Behavior when the "this" value is a Date object describing an invald date
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
---*/

var date = new Date(NaN);
var expected, result;

result = date.setFullYear(2016);

expected = new Date(2016, 0).getTime();
assert.sameValue(result, expected, 'return value (year)');
assert.sameValue(
  date.getTime(), expected, '[[DateValue]] internal slot (year)'
);

date = new Date(NaN);

result = date.setFullYear(2016, 6);

expected = new Date(2016, 6).getTime();
assert.sameValue(result, expected, 'return value (month)');
assert.sameValue(
  date.getTime(), expected, '[[DateValue]] internal slot (month)'
);

date = new Date(NaN);

result = date.setFullYear(2016, 6, 7);

expected = new Date(2016, 6, 7).getTime();
assert.sameValue(result, expected, 'return value (date)');
assert.sameValue(
  date.getTime(), expected, '[[DateValue]] internal slot (month)'
);
