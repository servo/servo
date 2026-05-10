// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-date.prototype.setutcmonth
description: >
  Read [[DateValue]] and then call ToNumber when stored time-value is invalid.
info: |
  Date.prototype.setUTCMonth ( month [ , date ] )

  ...
  3. Let t be dateObject.[[DateValue]].
  4. Let m be ? ToNumber(month).
  5. If date is present, let dt be ? ToNumber(date).
  6. If t is NaN, return NaN.
  ...
---*/

var dt = new Date(NaN);

var valueOfCalled = 0;

var value = {
  valueOf() {
    valueOfCalled++;
    dt.setTime(0);
    return 1;
  }
};

var result = dt.setUTCMonth(value);

assert.sameValue(valueOfCalled, 1, "valueOf called exactly once");

assert.sameValue(result, NaN, "result is NaN");

assert.sameValue(dt.getTime(), 0, "time updated in valueOf");
