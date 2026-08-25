// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.round
description: Circumstances where rounding is a no-op
features: [Temporal]
---*/

const instance = new Temporal.ZonedDateTime(0n, "UTC");

const noopRoundingOperations = [
  [{ smallestUnit: "nanoseconds" }, "smallestUnit ns"],
  [{ smallestUnit: "nanoseconds", roundingIncrement: 1 }, "round to 1 ns"],
];
for (const [options, descr] of noopRoundingOperations) {
  const result = instance.round(options);
  assert.notSameValue(result, instance, "rounding result should be a new object");
  assert.sameValue(result.epochNanoseconds, instance.epochNanoseconds, "instant should be unchanged");
}
