// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-date.prototype.setminutes
description: >
  Read [[DateValue]] and then call ToNumber when stored time-value is invalid.
info: |
  Date.prototype.setMinutes ( min [ , sec [ , ms ] ] )

  ...
  3. Let t be dateObject.[[DateValue]].
  4. Let m be ? ToNumber(min).
  5. If sec is present, let s be ? ToNumber(sec).
  6. If ms is present, let milli be ? ToNumber(ms).
  7. If t is NaN, return NaN.
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

var result = dt.setMinutes(value);

assert.sameValue(valueOfCalled, 1, "valueOf called exactly once");

assert.sameValue(result, NaN, "result is NaN");

assert.sameValue(dt.getTime(), 0, "time updated in valueOf");
