// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.until
description: >
  Rounding up does not add a spurious extra smallestUnit when the difference has
  zero of the smallestUnit
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const start = new Temporal.PlainYearMonth(2012, 1);

for (const roundingMode of ["ceil", "floor", "expand", "halfCeil", "halfFloor", "halfEven", "halfExpand", "halfTrunc", "trunc"]) {
  TemporalHelpers.assertDuration(
    start.until(new Temporal.PlainYearMonth(2013, 1),
      { smallestUnit: "months", largestUnit: "years", roundingMode, roundingIncrement: 2 }),
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    `P1Y months..years ${roundingMode}`
  );

  TemporalHelpers.assertDuration(
    start.until(new Temporal.PlainYearMonth(2011, 1),
      { smallestUnit: "months", largestUnit: "years", roundingMode, roundingIncrement: 2 }),
    -1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    `-P1Y months..years ${roundingMode}`
  );
}
