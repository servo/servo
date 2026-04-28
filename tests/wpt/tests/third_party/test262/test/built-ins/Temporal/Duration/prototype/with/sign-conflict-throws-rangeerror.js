// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.with
description: Throw RangeError if the resulting duration has mixed signs
info: |
  24. Return ? CreateTemporalDuration(years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds).
features: [Temporal]
---*/

const d1 = new Temporal.Duration(1, 2, 3, 4, 5, 6, 7, 8, 9, 10);
const d2 = new Temporal.Duration(-1, -2, -3, -4, -5, -6, -7, -8, -9, -10);
const fields = ["years", "months", "weeks", "days", "hours", "minutes", "seconds", "milliseconds", "microseconds", "nanoseconds"];

fields.forEach((field) => {
  assert.throws(
    RangeError,
    () => d1.with({ [field]: -1 }),
    `sign in argument { ${field}: -1 } conflicting with sign of duration should throw RangeError`
  );

  assert.throws(
    RangeError,
    () => d2.with({ [field]: 1 }),
    `sign in argument { ${field}: 1 } conflicting with sign of duration should throw RangeError`
  );
});
