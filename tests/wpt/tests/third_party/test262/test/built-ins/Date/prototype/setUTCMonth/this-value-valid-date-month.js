// Copyright (C) 2025 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.setutcmonth
description: Return value for valid dates (setting month)
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

returnValue = date.setUTCMonth(3);

expected = Date.UTC(2016, 3);
assert.sameValue(
  returnValue, expected, 'month within unit boundary (return value)'
);
assert.sameValue(
  date.getTime(), expected, 'month within unit boundary ([[DateValue]] slot)'
);

returnValue = date.setUTCMonth(-1);

expected = Date.UTC(2015, 11);
assert.sameValue(
  returnValue, expected, 'month before time unit boundary (return value)'
);
assert.sameValue(
  date.getTime(),
  expected,
  'month before time unit boundary ([[DateValue]] slot)'
);

returnValue = date.setUTCMonth(12);

expected = Date.UTC(2016, 0);
assert.sameValue(
  returnValue, expected, 'month after time unit boundary (return value)'
);
assert.sameValue(
  date.getTime(),
  expected,
  'month after time unit boundary ([[DateValue]] slot)'
);
