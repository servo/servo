// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The [[Value]] property of the newly constructed object
    is set by following steps:
    1. Call ToNumber(year)
    2. Call ToNumber(month)
    3. If date is supplied use ToNumber(date)
    4. If hours is supplied use ToNumber(hours)
    5. If minutes is supplied use ToNumber(minutes)
    6. If seconds is supplied use ToNumber(seconds)
    7. If ms is supplied use ToNumber(ms)
esid: sec-date-year-month-date-hours-minutes-seconds-ms
description: 3 arguments, (year, month, date)
---*/

function PoisonedValueOf(val) {
  this.value = val;
  this.valueOf = function() {
    throw new Test262Error();
  };
  this.toString = function() {};
}

assert.throws(Test262Error, () => {
  new Date(new PoisonedValueOf(1), new PoisonedValueOf(2), new PoisonedValueOf(3));
}, '`new Date(new PoisonedValueOf(1), new PoisonedValueOf(2), new PoisonedValueOf(3))` throws a Test262Error exception');

assert.throws(Test262Error, () => {
  new Date(1, new PoisonedValueOf(2), new PoisonedValueOf(3));
}, '`new Date(1, new PoisonedValueOf(2), new PoisonedValueOf(3))` throws a Test262Error exception');

assert.throws(Test262Error, () => {
  new Date(1, 2, new PoisonedValueOf(3));
}, '`new Date(1, 2, new PoisonedValueOf(3))` throws a Test262Error exception');
