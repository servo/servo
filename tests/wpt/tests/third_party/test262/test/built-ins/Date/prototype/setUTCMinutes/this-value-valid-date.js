// Copyright (C) 2025 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.setutcminutes
description: Return value for valid dates
info: |
  1. Let dateObject be the this value.
  2. Perform ? RequireInternalSlot(dateObject, [[DateValue]]).
  3. Let t be dateObject.[[DateValue]].
  4. Let m be ? ToNumber(min).
  5. If sec is present, let s be ? ToNumber(sec).
  6. If ms is present, let milli be ? ToNumber(ms).
  7. If t is NaN, return NaN.
  8. If sec is not present, let s be SecFromTime(t).
  9. If ms is not present, let milli be msFromTime(t).
  10. Let date be MakeDate(Day(t), MakeTime(HourFromTime(t), m, s, milli)).
  11. Let v be TimeClip(date).
  12. Set dateObject.[[DateValue]] to v.
  13. Return v.
---*/

var date = new Date(Date.UTC(2016, 6, 1));
var returnValue, expected;

returnValue = date.setUTCMinutes(23);

expected = Date.UTC(2016, 6, 1, 0, 23);
assert.sameValue(
  returnValue, expected, 'minute within unit boundary (return value)'
);
assert.sameValue(
  date.getTime(), expected, 'minute within unit boundary ([[DateValue]] slot)'
);

returnValue = date.setUTCMinutes(-1);

expected = Date.UTC(2016, 5, 30, 23, 59);
assert.sameValue(
  returnValue, expected, 'minute before time unit boundary (return value)'
);
assert.sameValue(
  date.getTime(),
  expected,
  'minute before time unit boundary ([[DateValue]] slot)'
);

returnValue = date.setUTCMinutes(60);

expected = Date.UTC(2016, 6, 1);
assert.sameValue(
  returnValue, expected, 'minute after time unit boundary (return value)'
);
assert.sameValue(
  date.getTime(),
  expected,
  'minute after time unit boundary ([[DateValue]] slot)'
);

date = new Date(Date.UTC(2016, 6, 1));

returnValue = date.setUTCMinutes(0, 45);

expected = Date.UTC(2016, 6, 1, 0, 0, 45);
assert.sameValue(
  returnValue, expected, 'second within unit boundary (return value)'
);
assert.sameValue(
  date.getTime(), expected, 'second within unit boundary ([[DateValue]] slot)'
);

returnValue = date.setUTCMinutes(0, -1);

expected = Date.UTC(2016, 5, 30, 23, 59, 59);
assert.sameValue(
  returnValue, expected, 'second before time unit boundary (return value)'
);
assert.sameValue(
  date.getTime(),
  expected,
  'second before time unit boundary ([[DateValue]] slot)'
);

returnValue = date.setUTCMinutes(0, 60);

expected = Date.UTC(2016, 5, 30, 23, 1);
assert.sameValue(
  returnValue, expected, 'second after time unit boundary (return value)'
);
assert.sameValue(
  date.getTime(),
  expected,
  'second after time unit boundary ([[DateValue]] slot)'
);

date = new Date(Date.UTC(2016, 6, 1));

returnValue = date.setUTCMinutes(0, 0, 345);

expected = Date.UTC(2016, 6, 1, 0, 0, 0, 345);
assert.sameValue(
  returnValue, expected, 'millisecond within unit boundary (return value)'
);
assert.sameValue(
  date.getTime(),
  expected,
  'millisecond within unit boundary ([[DateValue]] slot)'
);

returnValue = date.setUTCMinutes(0, 0, -1);

expected = Date.UTC(2016, 5, 30, 23, 59, 59, 999);
assert.sameValue(
  returnValue, expected, 'millisecond before time unit boundary (return value)'
);
assert.sameValue(
  date.getTime(),
  expected,
  'millisecond before time unit boundary ([[DateValue]] slot)'
);

returnValue = date.setUTCMinutes(0, 0, 1000);

expected = Date.UTC(2016, 5, 30, 23, 0, 1);
assert.sameValue(
  returnValue, expected, 'millisecond after time unit boundary (return value)'
);
assert.sameValue(
  date.getTime(),
  expected,
  'millisecond after time unit boundary ([[DateValue]] slot)'
);
