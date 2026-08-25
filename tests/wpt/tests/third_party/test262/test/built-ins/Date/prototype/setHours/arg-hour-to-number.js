// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.sethours
description: Type coercion of provided "hour"
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

returnValue = date.setHours(arg);

assert.sameValue(callCount, 1, 'invoked exactly once');
assert.sameValue(args.length, 0, 'invoked without arguments');
assert.sameValue(thisValue, arg, '"this" value');
assert.sameValue(
  returnValue,
  new Date(2016, 6, 1, 2).getTime(),
  'application of specified value'
);

returnValue = date.setHours(null);

assert.sameValue(returnValue, new Date(2016, 6, 1, 0).getTime(), 'null');

returnValue = date.setHours(true);

assert.sameValue(returnValue, new Date(2016, 6, 1, 1).getTime(), 'true');

returnValue = date.setHours(false);

assert.sameValue(returnValue, new Date(2016, 6, 1, 0).getTime(), 'false');

returnValue = date.setHours('   +00200.000E-0002	');

assert.sameValue(returnValue, new Date(2016, 6, 1, 2).getTime(), 'string');

returnValue = date.setHours();

assert.sameValue(returnValue, NaN, 'undefined');
