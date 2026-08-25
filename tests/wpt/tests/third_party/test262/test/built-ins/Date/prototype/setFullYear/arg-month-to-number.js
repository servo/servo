// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.setfullyear
description: Type coercion of provided "month"
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

var date = new Date(2016, 6, 7, 11, 36, 23, 2);
var callCount = 0;
var arg = {
  valueOf: function() {
    args = arguments;
    thisValue = this;
    callCount += 1;
    return 2;
  }
};
var args, thisValue, returnValue;

returnValue = date.setFullYear(2016, arg);

assert.sameValue(callCount, 1, 'invoked exactly once');
assert.sameValue(args.length, 0, 'invoked without arguments');
assert.sameValue(thisValue, arg, '"this" value');
assert.sameValue(
  returnValue,
  new Date(2016, 2, 7, 11, 36, 23, 2).getTime(),
  'application of specified value'
);

returnValue = date.setFullYear(2016, null);

assert.sameValue(
  returnValue,
  new Date(2016, 0, 7, 11, 36, 23, 2).getTime(),
  'null'
);

returnValue = date.setFullYear(2016, true);

assert.sameValue(
  returnValue,
  new Date(2016, 1, 7, 11, 36, 23, 2).getTime(),
  'true'
);

returnValue = date.setFullYear(2016, false);

assert.sameValue(
  returnValue,
  new Date(2016, 0, 7, 11, 36, 23, 2).getTime(),
  'false'
);

returnValue = date.setFullYear(2016, '   +00200.000E-0002	');

assert.sameValue(
  returnValue,
  new Date(2016, 2, 7, 11, 36, 23, 2).getTime(),
  'string'
);

returnValue = date.setFullYear(2016, undefined);

assert.sameValue(returnValue, NaN, 'undefined');
