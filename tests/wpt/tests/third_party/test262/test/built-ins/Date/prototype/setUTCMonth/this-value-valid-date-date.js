// Copyright (C) 2025 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.setutcmonth
description: Return value for valid dates (setting date)
info: |
  1. Let dateObject be the this value.
  2. Perform ? RequireInternalSlot(dateObject, [[DateValue]]).
  3. Let t be dateObject.[[DateValue]].
  4. Let m be ? ToNumber(month).
  5. If date is present, let dt be ? ToNumber(date).
  6. If t is NaN, return NaN.
  7. If date is not present, let dt be DateFromTime(t).
  8. Let newDate be MakeDate(MakeDay(YearFromTime(t), m, dt), TimeWithinDay(t)).
  9. Let v be TimeClip(newDate).
  10. Set dateObject.[[DateValue]] to v.
  11. Return v.
---*/

var date = new Date(Date.UTC(2016, 6));
var returnValue, expected;

returnValue = date.setUTCMonth(6, 6);

expected = Date.UTC(2016, 6, 6);
assert.sameValue(
  returnValue, expected, 'date within unit boundary (return value)'
);
assert.sameValue(
  date.getTime(), expected, 'date within unit boundary ([[DateValue]] slot)'
);

returnValue = date.setUTCMonth(6, 0);

expected = Date.UTC(2016, 5, 30);
assert.sameValue(
  returnValue, expected, 'date before time unit boundary (return value)'
);
assert.sameValue(
  date.getTime(),
  expected,
  'date before time unit boundary ([[DateValue]] slot)'
);

returnValue = date.setUTCMonth(5, 31);

expected = Date.UTC(2016, 6, 1);
assert.sameValue(
  returnValue, expected, 'date after time unit boundary (return value)'
);
assert.sameValue(
  date.getTime(),
  expected,
  'date after time unit boundary ([[DateValue]] slot)'
);
