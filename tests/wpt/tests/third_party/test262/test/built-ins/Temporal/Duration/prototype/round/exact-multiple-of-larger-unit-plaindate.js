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

const relativeTo = new Temporal.PlainDate(2012, 1, 1);

for (const roundingMode of ["ceil", "floor", "expand", "halfCeil", "halfFloor", "halfEven", "halfExpand", "halfTrunc", "trunc"]) {
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
