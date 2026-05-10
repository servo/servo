// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.setminutes
description: Type coercion of provided "sec"
info: |
  1. Let t be LocalTime(? thisTimeValue(this value)).
  2. Let m be ? ToNumber(min).
  3. If sec is not specified, let s be SecFromTime(t); otherwise, let s be ?
     ToNumber(sec).
  4. If ms is not specified, let milli be msFromTime(t); otherwise, let milli
     be ? ToNumber(ms).
  5. Let date be MakeDate(Day(t), MakeTime(HourFromTime(t), m, s, milli)).
  6. Let u be TimeClip(UTC(date)).
  7. Set the [[DateValue]] internal slot of this Date object to u.
  8. Return u.
---*/

var date = new Date(2016, 6);
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

returnValue = date.setMinutes(0, arg);

assert.sameValue(callCount, 1, 'invoked exactly once');
assert.sameValue(args.length, 0, 'invoked without arguments');
assert.sameValue(thisValue, arg, '"this" value');
assert.sameValue(
  returnValue,
  new Date(2016, 6, 1, 0, 0, 2).getTime(),
  'application of specified value'
);

returnValue = date.setMinutes(0, null);

assert.sameValue(returnValue, new Date(2016, 6, 1).getTime(), 'null');

returnValue = date.setMinutes(0, true);

assert.sameValue(returnValue, new Date(2016, 6, 1, 0, 0, 1).getTime(), 'true');

returnValue = date.setMinutes(0, false);

assert.sameValue(returnValue, new Date(2016, 6, 1).getTime(), 'false');

returnValue = date.setMinutes(0, '   +00200.000E-0002	');

assert.sameValue(
  returnValue, new Date(2016, 6, 1, 0, 0, 2).getTime(), 'string'
);

returnValue = date.setMinutes(0, undefined);

assert.sameValue(returnValue, NaN, 'undefined');
