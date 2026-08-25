// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: >
  Circumstances where rounding is a no-op, return a new but equal duration
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const plainRelativeTo = new Temporal.PlainDate(2000, 1, 1, "iso8601");
const zonedRelativeTo = new Temporal.ZonedDateTime(0n, "UTC", "iso8601");

const d = new Temporal.Duration(0, 0, 0, 0, 23, 59, 59, 999, 999, 997);

const noopRoundingOperations = [
  [d, { smallestUnit: "nanoseconds" }, "smallestUnit ns"],
  [d, { smallestUnit: "nanoseconds", relativeTo: plainRelativeTo }, "smallestUnit ns and plain relativeTo"],
  [d, { smallestUnit: "nanoseconds", relativeTo: zonedRelativeTo }, "smallestUnit ns and zoned relativeTo"],
  [d, { smallestUnit: "nanoseconds", roundingIncrement: 1 }, "round to 1 ns"],
  // No balancing because largestUnit is already the largest unit and no time units overflow:
  [d, { largestUnit: "hours" }, "largestUnit hours"],
  // Unless relativeTo is ZonedDateTime, no-op is still possible with days>0:
  [new Temporal.Duration(0, 0, 0, 1), { smallestUnit: "nanoseconds" }, "days>0 and smallestUnit ns"],
  [new Temporal.Duration(0, 0, 0, 1), { smallestUnit: "nanoseconds", relativeTo: plainRelativeTo }, "days>0, smallestUnit ns, and plain relativeTo"],
];
for (const [duration, options, descr] of noopRoundingOperations) {
  const result = duration.round(options);
  assert.notSameValue(result, duration, "rounding result should be a new object");
  TemporalHelpers.assertDurationsEqual(result, duration, `rounding should be a no-op with ${descr}`);

  const negDuration = duration.negated();
  const negResult = negDuration.round(options);
  assert.notSameValue(negResult, negDuration, "rounding result should be a new object (negative)");
  TemporalHelpers.assertDurationsEqual(negResult, negDuration, `rounding should be a no-op with ${descr} (negative)`);
}
