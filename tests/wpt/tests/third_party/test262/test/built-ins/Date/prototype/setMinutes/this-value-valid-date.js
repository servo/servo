// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.setminutes
description: Return value for valid dates
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

var date = new Date(2016, 6, 1);
var returnValue, expected;

returnValue = date.setMinutes(23);

expected = new Date(2016, 6, 1, 0, 23).getTime();
assert.sameValue(
  returnValue, expected, 'minute within unit boundary (return value)'
);
assert.sameValue(
  date.getTime(), expected, 'minute within unit boundary ([[DateValue]] slot)'
);

returnValue = date.setMinutes(-1);

expected = new Date(2016, 5, 30, 23, 59).getTime();
assert.sameValue(
  returnValue, expected, 'minute before time unit boundary (return value)'
);
assert.sameValue(
  date.getTime(),
  expected,
  'minute before time unit boundary ([[DateValue]] slot)'
);

returnValue = date.setMinutes(60);

expected = new Date(2016, 6, 1).getTime();
assert.sameValue(
  returnValue, expected, 'minute after time unit boundary (return value)'
);
assert.sameValue(
  date.getTime(),
  expected,
  'minute after time unit boundary ([[DateValue]] slot)'
);

date = new Date(2016, 6, 1);

returnValue = date.setMinutes(0, 45);

expected = new Date(2016, 6, 1, 0, 0, 45).getTime();
assert.sameValue(
  returnValue, expected, 'second within unit boundary (return value)'
);
assert.sameValue(
  date.getTime(), expected, 'second within unit boundary ([[DateValue]] slot)'
);

returnValue = date.setMinutes(0, -1);

expected = new Date(2016, 5, 30, 23, 59, 59).getTime();
assert.sameValue(
  returnValue, expected, 'second before time unit boundary (return value)'
);
assert.sameValue(
  date.getTime(),
  expected,
  'second before time unit boundary ([[DateValue]] slot)'
);

returnValue = date.setMinutes(0, 60);

expected = new Date(2016, 5, 30, 23, 1).getTime();
assert.sameValue(
  returnValue, expected, 'second after time unit boundary (return value)'
);
assert.sameValue(
  date.getTime(),
  expected,
  'second after time unit boundary ([[DateValue]] slot)'
);

date = new Date(2016, 6, 1);

returnValue = date.setMinutes(0, 0, 345);

expected = new Date(2016, 6, 1, 0, 0, 0, 345).getTime();
assert.sameValue(
  returnValue, expected, 'millisecond within unit boundary (return value)'
);
assert.sameValue(
  date.getTime(),
  expected,
  'millisecond within unit boundary ([[DateValue]] slot)'
);

returnValue = date.setMinutes(0, 0, -1);

expected = new Date(2016, 5, 30, 23, 59, 59, 999).getTime();
assert.sameValue(
  returnValue, expected, 'millisecond before time unit boundary (return value)'
);
assert.sameValue(
  date.getTime(),
  expected,
  'millisecond before time unit boundary ([[DateValue]] slot)'
);

returnValue = date.setMinutes(0, 0, 1000);

expected = new Date(2016, 5, 30, 23, 0, 1).getTime();
assert.sameValue(
  returnValue, expected, 'millisecond after time unit boundary (return value)'
);
assert.sameValue(
  date.getTime(),
  expected,
  'millisecond after time unit boundary ([[DateValue]] slot)'
);
