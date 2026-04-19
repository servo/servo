// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.subtract
description: RangeError thrown if result is outside representable range
features: [Temporal]
---*/

const fields = ["hours", "minutes", "seconds", "milliseconds", "microseconds", "nanoseconds"];

const earliest = Temporal.Instant.fromEpochNanoseconds(-8640000_000_000_000_000_000n);

fields.forEach((field) => {
  assert.throws(
    RangeError,
    () => earliest.subtract({ [field]: 1 }),
    `subtracting ${field} with result out of range (negative)`
  );
});

const latest = Temporal.Instant.fromEpochNanoseconds(8640000_000_000_000_000_000n);

fields.forEach((field) => {
  assert.throws(
    RangeError,
    () => latest.subtract({ [field]: -1 }),
    `subtracting ${field} with result out of range (positive)`
  );
});
