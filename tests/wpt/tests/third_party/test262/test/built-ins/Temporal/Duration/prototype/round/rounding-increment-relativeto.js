// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: Test a specific buggy case from temporal_rs
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const plainRelativeTo = new Temporal.PlainDate(2020, 1, 1);
const zonedRelativeTo = new Temporal.ZonedDateTime(0n, "UTC");

for (const relativeTo of [plainRelativeTo, zonedRelativeTo]) {
  TemporalHelpers.assertDuration(new Temporal.Duration(0, 0, 1, 0, 168).round({
    smallestUnit: "weeks",
    roundingIncrement: 2,
    relativeTo
  }), 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, `1w168h to 2w with relativeTo ${relativeTo}`);
}

for (const relativeTo of [undefined, plainRelativeTo, zonedRelativeTo]) {
  TemporalHelpers.assertDuration(new Temporal.Duration(0, 0, 0, 0, 48).round({
    smallestUnit: "days",
    roundingIncrement: 2,
    relativeTo
  }), 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, `48h to 2d with relativeTo ${relativeTo}`);
}

// Other specific cases where the relativeTo date seems to be significant,
// unlike the above cases:
TemporalHelpers.assertDuration(new Temporal.Duration(0, 1, 0, 30).round({
  smallestUnit: "months",
  roundingIncrement: 2,
  relativeTo: new Temporal.PlainDate(1970, 3, 1)
}), 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, `1m30d to 2m with relativeTo 1970-03-01`);
TemporalHelpers.assertDuration(new Temporal.Duration(0, 1, 0, 30).round({
  smallestUnit: "months",
  roundingIncrement: 2,
  relativeTo: new Temporal.PlainDate(1970, 7, 31)
}), 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, `1m30d to 2m with relativeTo 1970-07-31`);
