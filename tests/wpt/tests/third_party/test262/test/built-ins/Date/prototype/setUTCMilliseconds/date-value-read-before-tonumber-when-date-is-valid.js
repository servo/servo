// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-date.prototype.setutcmilliseconds
description: >
  Read [[DateValue]] and then call ToNumber when stored time-value is valid.
info: |
  Date.prototype.setUTCMilliseconds ( ms )

  ...
  3. Let t be dateObject.[[DateValue]].
  4. Set ms to ? ToNumber(ms).
  5. If t is NaN, return NaN.
  6. Let time be MakeTime(HourFromTime(t), MinFromTime(t), SecFromTime(t), ms).
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

var result = dt.setUTCMilliseconds(value);

assert.sameValue(valueOfCalled, 1, "valueOf called exactly once");

assert.notSameValue(result, NaN, "result is not NaN");

assert.sameValue(result, dt.getTime(), "result is equal to getTime");

assert.sameValue(dt.getUTCMilliseconds(), 1, "date value correctly updated");
