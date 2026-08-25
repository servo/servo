// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-date.prototype.setfullyear
description: >
  Read [[DateValue]] and then call ToNumber when stored time-value is valid.
info: |
  Date.prototype.setFullYear ( year [ , month [ , date ] ] )

  ...
  3. Let t be dateObject.[[DateValue]].
  4. Let y be ? ToNumber(year).
  5. If t is NaN, set t to +0𝔽; otherwise, set t to LocalTime(t).
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

var result = dt.setFullYear(value);

assert.sameValue(valueOfCalled, 1, "valueOf called exactly once");

assert.notSameValue(result, NaN, "result is not NaN");

assert.sameValue(result, dt.getTime(), "result is equal to getTime");

assert.sameValue(dt.getFullYear(), 1, "date value correctly updated");
