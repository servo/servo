// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-date.prototype.setutcfullyear
description: >
  Read [[DateValue]] and then call ToNumber when stored time-value is invalid.
info: |
  Date.prototype.setUTCFullYear ( year [ , month [ , date ] ] )

  ...
  3. Let t be dateObject.[[DateValue]].
  4. If t is NaN, set t to +0𝔽.
  5. Let y be ? ToNumber(year).
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

var result = dt.setUTCFullYear(value);

assert.sameValue(valueOfCalled, 1, "valueOf called exactly once");

assert.notSameValue(result, NaN, "result is not NaN");

assert.sameValue(result, dt.getTime(), "result is equal to getTime");

assert.sameValue(dt.getUTCFullYear(), 1, "date value correctly updated");
