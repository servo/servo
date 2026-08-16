// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: >
  Rounding with largestUnit "year" with relativeTo near the earlier date limit
  correctly rounds the day even though year boundaries fall outside the valid
  range
info: Firefox bug https://bugzilla.mozilla.org/show_bug.cgi?id=2036259
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const relativeTo = new Temporal.PlainDate(-271821, 5, 19);
const duration = new Temporal.Duration(0, 0, 0, 0, -23);

const result = duration.round({
  largestUnit: "year",
  smallestUnit: "day",
  roundingMode: "expand",
  relativeTo,
});

TemporalHelpers.assertDuration(result, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0,
  "-23 hours rounds to -1 day near minimum date");
