// Copyright (C) 2025 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.setutcmilliseconds
description: Return value for valid dates
info: |
  1. Let dateObject be the this value.
  2. Perform ? RequireInternalSlot(dateObject, [[DateValue]]).
  3. Let t be dateObject.[[DateValue]].
  4. Set ms to ? ToNumber(ms).
  5. If t is NaN, return NaN.
  6. Let time be MakeTime(HourFromTime(t), MinFromTime(t), SecFromTime(t), ms).
  7. Let v be TimeClip(MakeDate(Day(t), time)).
  8. Set dateObject.[[DateValue]] to v.
  9. Return v.
---*/

var date = new Date(Date.UTC(2016, 6));
var returnValue, expected;

returnValue = date.setUTCMilliseconds(333);

expected = Date.UTC(2016, 6, 1, 0, 0, 0, 333);
assert.sameValue(
  returnValue, expected, 'within unit boundary (return value)'
);
assert.sameValue(
  date.getTime(), expected, 'within unit boundary ([[DateValue]] slot)'
);

returnValue = date.setUTCMilliseconds(-1);

expected = Date.UTC(2016, 5, 30, 23, 59, 59, 999);
assert.sameValue(
  returnValue, expected, 'before time unit boundary (return value)'
);
assert.sameValue(
  date.getTime(), expected, 'before time unit boundary ([[DateValue]] slot)'
);

returnValue = date.setUTCMilliseconds(1000);

expected = Date.UTC(2016, 6, 1);
assert.sameValue(
  returnValue, expected, 'after time unit boundary (return value)'
);
assert.sameValue(
  date.getTime(), expected, 'after time unit boundary ([[DateValue]] slot)'
);
