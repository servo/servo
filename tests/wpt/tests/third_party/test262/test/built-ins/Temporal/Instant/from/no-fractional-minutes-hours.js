// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.from
description: Fractional minutes or hours in time string should throw RangeError
features: [Temporal]
---*/

const invalidStrings = [
  ["2025-04-03T05:07.123[UTC]", "Fractional minutes"],
  ["2025-04-03T12.5[UTC]", "Fractional hours"],
];

for (const [arg, description] of invalidStrings) {
  assert.throws(
    RangeError,
      () => Temporal.Instant.from(arg),
    `${description} not allowed in time string`
  );
}
