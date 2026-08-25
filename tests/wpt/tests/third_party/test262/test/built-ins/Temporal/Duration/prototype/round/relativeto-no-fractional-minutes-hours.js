// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: Fractional minutes or hours in time string in relativeTo option should throw RangeError
features: [Temporal]
---*/

const instance = new Temporal.Duration(1, 0, 0, 0, 24)

const invalidStrings = [
  ["2025-04-03T05:07.123", "Fractional minutes"],
  ["2025-04-03T12.5", "Fractional hours"],
];

for (const [arg, description] of invalidStrings) {
  assert.throws(
    RangeError,
    () => instance.round({ largestUnit: "months", relativeTo: arg }),
    `${description} not allowed in time string`
  );
}
