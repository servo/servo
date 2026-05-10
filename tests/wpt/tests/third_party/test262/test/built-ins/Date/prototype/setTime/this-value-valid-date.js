// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.settime
description: Return value for valid dates
info: |
  1. Let t be LocalTime(? thisTimeValue(this value)).
  2. Let dt be ? ToNumber(date).
  3. Let newDate be MakeDate(MakeDay(YearFromTime(t), MonthFromTime(t), dt),
     TimeWithinDay(t)).
  4. Let u be TimeClip(UTC(newDate)).
  5. Set the [[DateValue]] internal slot of this Date object to u.
  6. Return u.
---*/

var date = new Date(2016, 6);
var returnValue;

returnValue = date.setTime(6);

assert.sameValue(
  returnValue, 6, 'within unit boundary (return value)'
);
assert.sameValue(
  date.getTime(), 6, 'within unit boundary ([[DateValue]] slot)'
);
