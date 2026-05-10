// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.setmonth
description: Return value for valid dates (setting month)
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

var date = new Date(2016, 6);
var returnValue, expected;

returnValue = date.setMonth(3);

expected = new Date(2016, 3).getTime();
assert.sameValue(
  returnValue, expected, 'month within unit boundary (return value)'
);
assert.sameValue(
  date.getTime(), expected, 'month within unit boundary ([[DateValue]] slot)'
);

returnValue = date.setMonth(-1);

expected = new Date(2015, 11).getTime();
assert.sameValue(
  returnValue, expected, 'month before time unit boundary (return value)'
);
assert.sameValue(
  date.getTime(),
  expected,
  'month before time unit boundary ([[DateValue]] slot)'
);

returnValue = date.setMonth(12);

expected = new Date(2016, 0).getTime();
assert.sameValue(
  returnValue, expected, 'month after time unit boundary (return value)'
);
assert.sameValue(
  date.getTime(),
  expected,
  'month after time unit boundary ([[DateValue]] slot)'
);
