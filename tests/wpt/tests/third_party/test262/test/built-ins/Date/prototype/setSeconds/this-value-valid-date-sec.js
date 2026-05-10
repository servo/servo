// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.setseconds
description: Return value for valid dates (setting sec)
info: |
  1. Let t be LocalTime(? thisTimeValue(this value)).
  2. Let s be ? ToNumber(sec).
  3. If ms is not specified, let milli be msFromTime(t); otherwise, let milli
     be ? ToNumber(ms).
  4. Let date be MakeDate(Day(t), MakeTime(HourFromTime(t), MinFromTime(t), s,
     milli)).
  5. Let u be TimeClip(UTC(date)).
  6. Set the [[DateValue]] internal slot of this Date object to u.
  7. Return u.
---*/

var date = new Date(2016, 6);
var returnValue, expected;

returnValue = date.setSeconds(45);

expected = new Date(2016, 6, 1, 0, 0, 45).getTime();
assert.sameValue(
  returnValue, expected, 'second within unit boundary (return value)'
);
assert.sameValue(
  date.getTime(), expected, 'second within unit boundary ([[DateValue]] slot)'
);

returnValue = date.setSeconds(-1);

expected = new Date(2016, 5, 30, 23, 59, 59).getTime();
assert.sameValue(
  returnValue, expected, 'second before time unit boundary (return value)'
);
assert.sameValue(
  date.getTime(),
  expected,
  'second before time unit boundary ([[DateValue]] slot)'
);

returnValue = date.setSeconds(60);

expected = new Date(2016, 6).getTime();
assert.sameValue(
  returnValue, expected, 'second after time unit boundary (return value)'
);
assert.sameValue(
  date.getTime(),
  expected,
  'second after time unit boundary ([[DateValue]] slot)'
);
