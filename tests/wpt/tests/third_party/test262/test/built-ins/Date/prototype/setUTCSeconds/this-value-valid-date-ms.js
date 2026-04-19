// Copyright (C) 2025 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.setutcseconds
description: Return value for valid dates (setting ms)
info: |
  1. Let dateObject be the this value.
  2. Perform ? RequireInternalSlot(dateObject, [[DateValue]]).
  3. Let t be dateObject.[[DateValue]].
  4. Let s be ? ToNumber(sec).
  5. If ms is present, let milli be ? ToNumber(ms).
  6. If t is NaN, return NaN.
  7. If ms is not present, let milli be msFromTime(t).
  8. Let date be MakeDate(Day(t), MakeTime(HourFromTime(t), MinFromTime(t), s, milli)).
  9. Let v be TimeClip(date).
  10. Set dateObject.[[DateValue]] to v.
  11. Return v.
---*/

var date = new Date(Date.UTC(2016, 6));
var returnValue, expected;

returnValue = date.setUTCSeconds(0, 543);

expected = Date.UTC(2016, 6, 1, 0, 0, 0, 543);
assert.sameValue(
  returnValue, expected, 'millisecond within unit boundary (return value)'
);
assert.sameValue(
  date.getTime(),
  expected,
  'millisecond within unit boundary ([[DateValue]] slot)'
);

returnValue = date.setUTCSeconds(0, -1);

expected = Date.UTC(2016, 5, 30, 23, 59, 59, 999);
assert.sameValue(
  returnValue, expected, 'millisecond before time unit boundary (return value)'
);
assert.sameValue(
  date.getTime(),
  expected,
  'millisecond before time unit boundary ([[DateValue]] slot)'
);

returnValue = date.setUTCSeconds(0, 1000);

expected = Date.UTC(2016, 5, 30, 23, 59, 1, 0);
assert.sameValue(
  returnValue, expected, 'millisecond after time unit boundary (return value)'
);
assert.sameValue(
  date.getTime(),
  expected,
  'millisecond after time unit boundary ([[DateValue]] slot)'
);
