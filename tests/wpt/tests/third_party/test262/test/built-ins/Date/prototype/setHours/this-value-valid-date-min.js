// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.sethours
description: Return value for valid dates (setting min)
info: |
  1. Let t be LocalTime(? thisTimeValue(this value)).
  2. Let h be ? ToNumber(hour).
  3. If min is not specified, let m be MinFromTime(t); otherwise, let m be ?
     ToNumber(min).
  4. If sec is not specified, let s be SecFromTime(t); otherwise, let s be ?
     ToNumber(sec).
  5. If ms is not specified, let milli be msFromTime(t); otherwise, let milli
     be ? ToNumber(ms).
  6. Let date be MakeDate(Day(t), MakeTime(h, m, s, milli)).
  7. Let u be TimeClip(UTC(date)).
  8. Set the [[DateValue]] internal slot of this Date object to u.
  9. Return u.
---*/

var date = new Date(2016, 6);
var returnValue, expected;

returnValue = date.setHours(0, 23);

expected = new Date(2016, 6, 1, 0, 23).getTime();
assert.sameValue(
  returnValue, expected, 'minute within unit boundary (return value)'
);
assert.sameValue(
  date.getTime(), expected, 'minute within unit boundary ([[DateValue]] slot)'
);

returnValue = date.setHours(0, -1);

expected = new Date(2016, 5, 30, 23, 59).getTime();
assert.sameValue(
  returnValue, expected, 'minute before time unit boundary (return value)'
);
assert.sameValue(
  date.getTime(),
  expected,
  'minute before time unit boundary ([[DateValue]] slot)'
);

returnValue = date.setHours(0, 60);

expected = new Date(2016, 5, 30, 1).getTime();
assert.sameValue(
  returnValue, expected, 'minute after time unit boundary (return value)'
);
assert.sameValue(
  date.getTime(),
  expected,
  'minute after time unit boundary ([[DateValue]] slot)'
);
