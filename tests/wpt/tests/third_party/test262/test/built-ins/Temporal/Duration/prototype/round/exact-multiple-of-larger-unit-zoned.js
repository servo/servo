// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: >
  Rounding does not add a spurious extra smallestUnit when the difference has
  zero of the smallestUnit
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const relativeTo = Temporal.ZonedDateTime.from("2012-01-01T12:00:00+00:00[UTC]");

for (const roundingMode of ["ceil", "floor", "expand", "halfCeil", "halfFloor", "halfEven", "halfExpand", "halfTrunc", "trunc"]) {
  TemporalHelpers.assertDuration(
    new Temporal.Duration(0, 0, 0, 7).round(
      { smallestUnit: "days", largestUnit: "weeks", roundingMode, relativeTo }),
    0, 0, 1, 0, 0, 0, 0, 0, 0, 0,
    `P7D days..weeks ${roundingMode}`
  );
  TemporalHelpers.assertDuration(
    new Temporal.Duration(0, 0, 0, 31).round(
      { smallestUnit: "days", largestUnit: "months", roundingMode, relativeTo }),
    0, 1, 0, 0, 0, 0, 0, 0, 0, 0,
    `P31D days..months ${roundingMode}`
  );
  TemporalHelpers.assertDuration(
    new Temporal.Duration(1).round(
      { smallestUnit: "days", largestUnit: "years", roundingMode, relativeTo }),
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    `P1Y days..years ${roundingMode}`
  );
  TemporalHelpers.assertDuration(
    new Temporal.Duration(0, 0, 0, 31).round(
      { smallestUnit: "weeks", largestUnit: "months", roundingMode, relativeTo }),
    0, 1, 0, 0, 0, 0, 0, 0, 0, 0,
    `P31D weeks..months ${roundingMode}`
  );
  TemporalHelpers.assertDuration(
    new Temporal.Duration(1).round(
      { smallestUnit: "months", largestUnit: "years", roundingMode, relativeTo }),
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    `P1Y months..years ${roundingMode}`
  );

  TemporalHelpers.assertDuration(
    new Temporal.Duration(0, 0, 0, -7).round(
      { smallestUnit: "days", largestUnit: "weeks", roundingMode, relativeTo }),
    0, 0, -1, 0, 0, 0, 0, 0, 0, 0,
    `-P7D days..weeks ${roundingMode}`
  );
  TemporalHelpers.assertDuration(
    new Temporal.Duration(0, 0, 0, -31).round(
      { smallestUnit: "days", largestUnit: "months", roundingMode, relativeTo }),
    0, -1, 0, 0, 0, 0, 0, 0, 0, 0,
    `-P31D days..months ${roundingMode}`
  );
  TemporalHelpers.assertDuration(
    new Temporal.Duration(-1).round(
      { smallestUnit: "days", largestUnit: "years", roundingMode, relativeTo }),
    -1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    `-P1Y days..years ${roundingMode}`
  );
  TemporalHelpers.assertDuration(
    new Temporal.Duration(0, 0, 0, -31).round(
      { smallestUnit: "weeks", largestUnit: "months", roundingMode, relativeTo }),
    0, -1, 0, 0, 0, 0, 0, 0, 0, 0,
    `-P31D weeks..months ${roundingMode}`
  );
  TemporalHelpers.assertDuration(
    new Temporal.Duration(-1).round(
      { smallestUnit: "months", largestUnit: "years", roundingMode, relativeTo }),
    -1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    `-P1Y months..years ${roundingMode}`
  );
}
