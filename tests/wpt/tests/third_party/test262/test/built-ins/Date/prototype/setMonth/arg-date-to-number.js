// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.setmonth
description: Type coercion of provided "date"
info: |
  1. Let t be LocalTime(? thisTimeValue(this value)).
  2. Let m be ? ToNumber(month).
  3. If date is not specified, let dt be DateFromTime(t); otherwise, let dt be
     ? ToNumber(date).
  4. Let newDate be MakeDate(MakeDay(YearFromTime(t), m, dt),
     TimeWithinDay(t)).
  5. Let u be TimeClip(UTC(newDate)).
  6. Set the [[DateValue]] internal slot of this Date object to u.
  7. Return u.
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

returnValue = date.setMonth(6, arg);

assert.sameValue(callCount, 1, 'invoked exactly once');
assert.sameValue(args.length, 0, 'invoked without arguments');
assert.sameValue(thisValue, arg, '"this" value');
assert.sameValue(
  returnValue,
  new Date(2016, 6, 2, 11, 36, 23, 2).getTime(),
  'application of specified value'
);

returnValue = date.setMonth(6, null);

assert.sameValue(
  returnValue,
  new Date(2016, 6, 0, 11, 36, 23, 2).getTime(),
  'null'
);

returnValue = date.setMonth(6, true);

assert.sameValue(
  returnValue,
  new Date(2016, 6, 1, 11, 36, 23, 2).getTime(),
  'true'
);

returnValue = date.setMonth(6, false);

assert.sameValue(
  returnValue,
  new Date(2016, 6, 0, 11, 36, 23, 2).getTime(),
  'false'
);

returnValue = date.setMonth(6, '   +00200.000E-0002	');

assert.sameValue(
  returnValue,
  new Date(2016, 6, 2, 11, 36, 23, 2).getTime(),
  'string'
);

returnValue = date.setMonth(6, undefined);

assert.sameValue(returnValue, NaN, 'undefined');
