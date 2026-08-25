// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.equals
description: Fractional minutes or hours in time string should throw RangeError
features: [Temporal]
---*/

const instance = new Temporal.PlainDateTime(2000, 5, 2, 12, 34, 56, 987, 654, 321)

const invalidStrings = [
  ["2025-04-03T05:07.123", "Fractional minutes"],
  ["2025-04-03T12.5", "Fractional hours"],
];

for (const [arg, description] of invalidStrings) {
  assert.throws(
    RangeError,
      () => instance.equals(arg),
    `${description} not allowed in time string`
  );
}
