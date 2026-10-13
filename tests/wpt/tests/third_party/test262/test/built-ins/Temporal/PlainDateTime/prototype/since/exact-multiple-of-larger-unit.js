// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.since
description: >
  Rounding up does not add a spurious extra smallestUnit when the difference has
  zero of the smallestUnit
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const start = new Temporal.PlainDateTime(2012, 1, 1, 12);

for (const roundingMode of ["ceil", "floor", "expand", "halfCeil", "halfFloor", "halfEven", "halfExpand", "halfTrunc", "trunc"]) {
  TemporalHelpers.assertDuration(
    new Temporal.PlainDateTime(2012, 2, 1, 12).since(start,
      { smallestUnit: "weeks", largestUnit: "months", roundingMode }),
    0, 1, 0, 0, 0, 0, 0, 0, 0, 0,
    `P1M weeks..months ${roundingMode}`
  );
  TemporalHelpers.assertDuration(
    new Temporal.PlainDateTime(2013, 1, 1, 12).since(start,
      { smallestUnit: "months", largestUnit: "years", roundingMode }),
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    `P1Y months..years ${roundingMode}`
  );

  TemporalHelpers.assertDuration(
    new Temporal.PlainDateTime(2011, 12, 1, 12).since(start,
      { smallestUnit: "weeks", largestUnit: "months", roundingMode }),
    0, -1, 0, 0, 0, 0, 0, 0, 0, 0,
    `-P1M weeks..months ${roundingMode}`
  );
  TemporalHelpers.assertDuration(
    new Temporal.PlainDateTime(2011, 1, 1, 12).since(start,
      { smallestUnit: "months", largestUnit: "years", roundingMode }),
    -1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    `-P1Y months..years ${roundingMode}`
  );
}
