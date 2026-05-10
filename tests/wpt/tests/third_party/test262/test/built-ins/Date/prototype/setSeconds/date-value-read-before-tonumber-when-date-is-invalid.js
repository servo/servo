// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-date.prototype.setseconds
description: >
  Read [[DateValue]] and then call ToNumber when stored time-value is invalid.
info: |
  Date.prototype.setSeconds ( sec [ , ms ] )

  ...
  3. Let t be dateObject.[[DateValue]].
  4. Let s be ? ToNumber(sec).
  5. If ms is present, let milli be ? ToNumber(ms).
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

var result = dt.setSeconds(value);

assert.sameValue(valueOfCalled, 1, "valueOf called exactly once");

assert.sameValue(result, NaN, "result is NaN");

assert.sameValue(dt.getTime(), 0, "time updated in valueOf");
