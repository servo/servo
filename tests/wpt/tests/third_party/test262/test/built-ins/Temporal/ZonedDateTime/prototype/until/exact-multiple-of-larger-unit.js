// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.until
description: >
  Rounding up does not add a spurious extra smallestUnit when the difference has
  zero of the smallestUnit
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const start = Temporal.ZonedDateTime.from("2012-01-01T12:00:00+00:00[UTC]");

for (const roundingMode of ["ceil", "floor", "expand", "halfCeil", "halfFloor", "halfEven", "halfExpand", "halfTrunc", "trunc"]) {
  TemporalHelpers.assertDuration(
    start.until(Temporal.ZonedDateTime.from("2012-01-08T12:00:00+00:00[UTC]"),
      { smallestUnit: "days", largestUnit: "weeks", roundingMode }),
    0, 0, 1, 0, 0, 0, 0, 0, 0, 0,
    `P7D days..weeks ${roundingMode}`
  );
  TemporalHelpers.assertDuration(
    start.until(Temporal.ZonedDateTime.from("2012-02-01T12:00:00+00:00[UTC]"),
      { smallestUnit: "days", largestUnit: "months", roundingMode }),
    0, 1, 0, 0, 0, 0, 0, 0, 0, 0,
    `P1M days..months ${roundingMode}`
  );
  TemporalHelpers.assertDuration(
    start.until(Temporal.ZonedDateTime.from("2013-01-01T12:00:00+00:00[UTC]"),
      { smallestUnit: "days", largestUnit: "years", roundingMode }),
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    `P1Y days..years ${roundingMode}`
  );
  TemporalHelpers.assertDuration(
    start.until(Temporal.ZonedDateTime.from("2012-02-01T12:00:00+00:00[UTC]"),
      { smallestUnit: "weeks", largestUnit: "months", roundingMode }),
    0, 1, 0, 0, 0, 0, 0, 0, 0, 0,
    `P1M weeks..months ${roundingMode}`
  );
  TemporalHelpers.assertDuration(
    start.until(Temporal.ZonedDateTime.from("2013-01-01T12:00:00+00:00[UTC]"),
      { smallestUnit: "months", largestUnit: "years", roundingMode }),
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    `P1Y months..years ${roundingMode}`
  );

  TemporalHelpers.assertDuration(
    start.until(Temporal.ZonedDateTime.from("2011-12-25T12:00:00+00:00[UTC]"),
      { smallestUnit: "days", largestUnit: "weeks", roundingMode }),
    0, 0, -1, 0, 0, 0, 0, 0, 0, 0,
    `-P7D days..weeks ${roundingMode}`
  );
  TemporalHelpers.assertDuration(
    start.until(Temporal.ZonedDateTime.from("2011-12-01T12:00:00+00:00[UTC]"),
      { smallestUnit: "days", largestUnit: "months", roundingMode }),
    0, -1, 0, 0, 0, 0, 0, 0, 0, 0,
    `-P1M days..months ${roundingMode}`
  );
  TemporalHelpers.assertDuration(
    start.until(Temporal.ZonedDateTime.from("2011-01-01T12:00:00+00:00[UTC]"),
      { smallestUnit: "days", largestUnit: "years", roundingMode }),
    -1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    `-P1Y days..years ${roundingMode}`
  );
  TemporalHelpers.assertDuration(
    start.until(Temporal.ZonedDateTime.from("2011-12-01T12:00:00+00:00[UTC]"),
      { smallestUnit: "weeks", largestUnit: "months", roundingMode }),
    0, -1, 0, 0, 0, 0, 0, 0, 0, 0,
    `-P1M weeks..months ${roundingMode}`
  );
  TemporalHelpers.assertDuration(
    start.until(Temporal.ZonedDateTime.from("2011-01-01T12:00:00+00:00[UTC]"),
      { smallestUnit: "months", largestUnit: "years", roundingMode }),
    -1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    `-P1Y months..years ${roundingMode}`
  );
}
