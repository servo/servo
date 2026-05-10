// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.compare
description: Fractional minutes or hours in time string should throw RangeError
features: [Temporal]
---*/

const invalidStrings = [
  ["2025-04-03T05:07.123", "Fractional minutes"],
  ["2025-04-03T12.5", "Fractional hours"],
];

for (const [arg, description] of invalidStrings) {
  assert.throws(
    RangeError,
      () => Temporal.PlainYearMonth.compare(arg, new Temporal.PlainYearMonth(2025, 4)),
    `${description} not allowed in time string (first argument)`
  );
  assert.throws(
    RangeError,
      () => Temporal.PlainYearMonth.compare(new Temporal.PlainYearMonth(2025, 4), arg),
    `${description} not allowed in time string (second argument)`
  );
}
