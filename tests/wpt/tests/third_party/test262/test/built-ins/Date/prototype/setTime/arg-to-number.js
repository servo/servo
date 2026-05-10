// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.settime
description: Type coercion of provided argument
info: |
  1. Perform ? thisTimeValue(this value).
  2. Let t be ? ToNumber(time).
  3. Let v be TimeClip(t).
  4. Set the [[DateValue]] internal slot of this Date object to v.
  5. Return v.
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

returnValue = date.setTime(arg);

assert.sameValue(callCount, 1, 'invoked exactly once');
assert.sameValue(args.length, 0, 'invoked without arguments');
assert.sameValue(thisValue, arg, '"this" value');
assert.sameValue(returnValue, 2, 'application of specified value');

returnValue = date.setTime(null);

assert.sameValue(returnValue, 0, 'null');

returnValue = date.setTime(true);

assert.sameValue(returnValue, 1, 'true');

returnValue = date.setTime(false);

assert.sameValue(returnValue, 0, 'false');

returnValue = date.setTime('   +00200.000E-0002	');

assert.sameValue(returnValue, 2, 'string');

returnValue = date.setTime();

assert.sameValue(returnValue, NaN, 'undefined');
