// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-date.prototype.setutcmonth
description: >
  Read [[DateValue]] and then call ToNumber when stored time-value is valid.
info: |
  Date.prototype.setUTCMonth ( month [ , date ] )

  ...
  3. Let t be dateObject.[[DateValue]].
  4. Let m be ? ToNumber(month).
  5. If date is present, let dt be ? ToNumber(date).
  6. If t is NaN, return NaN.
  7. If date is not present, let dt be DateFromTime(t).
  ...
---*/

var dt = new Date(0);

var valueOfCalled = 0;

var value = {
  valueOf() {
    valueOfCalled++;
    dt.setTime(NaN);
    return 1;
  }
};

var result = dt.setUTCMonth(value);

assert.sameValue(valueOfCalled, 1, "valueOf called exactly once");

assert.notSameValue(result, NaN, "result is not NaN");

assert.sameValue(result, dt.getTime(), "result is equal to getTime");

assert.sameValue(dt.getUTCMonth(), 1, "date value correctly updated");
