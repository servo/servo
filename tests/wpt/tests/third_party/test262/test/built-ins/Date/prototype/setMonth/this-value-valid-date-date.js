// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.setmonth
description: Return value for valid dates (setting date)
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

returnValue = date.setMonth(6, 6);

expected = new Date(2016, 6, 6).getTime();
assert.sameValue(
  returnValue, expected, 'date within unit boundary (return value)'
);
assert.sameValue(
  date.getTime(), expected, 'date within unit boundary ([[DateValue]] slot)'
);

returnValue = date.setMonth(6, 0);

expected = new Date(2016, 5, 30).getTime();
assert.sameValue(
  returnValue, expected, 'date before time unit boundary (return value)'
);
assert.sameValue(
  date.getTime(),
  expected,
  'date before time unit boundary ([[DateValue]] slot)'
);

returnValue = date.setMonth(5, 31);

expected = new Date(2016, 6, 1).getTime();
assert.sameValue(
  returnValue, expected, 'date after time unit boundary (return value)'
);
assert.sameValue(
  date.getTime(),
  expected,
  'date after time unit boundary ([[DateValue]] slot)'
);
