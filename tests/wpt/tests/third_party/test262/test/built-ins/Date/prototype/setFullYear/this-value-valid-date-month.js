// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.setfullyear
description: Return value for valid dates (setting month)
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

var date = new Date(2016, 6);
var returnValue, expected;

returnValue = date.setFullYear(2016, 3);

expected = new Date(2016, 3).getTime();
assert.sameValue(
  returnValue, expected, 'month within unit boundary (return value)'
);
assert.sameValue(
  date.getTime(), expected, 'month within unit boundary ([[DateValue]] slot)'
);

returnValue = date.setFullYear(2016, -1);

expected = new Date(2015, 11).getTime();
assert.sameValue(
  returnValue, expected, 'month before time unit boundary (return value)'
);
assert.sameValue(
  date.getTime(),
  expected,
  'month before time unit boundary ([[DateValue]] slot)'
);

returnValue = date.setFullYear(2016, 12);

expected = new Date(2017, 0).getTime();
assert.sameValue(
  returnValue, expected, 'month after time unit boundary (return value)'
);
assert.sameValue(
  date.getTime(),
  expected,
  'month after time unit boundary ([[DateValue]] slot)'
);
