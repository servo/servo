// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.until
description: >
  Rounding with largestUnit "year" near the earlier date limit correctly rounds
  the day increment even though year boundaries fall outside the valid range
info: Firefox bug https://bugzilla.mozilla.org/show_bug.cgi?id=2036259
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const d1 = new Temporal.PlainDate(-271821, 5, 19);
const d2 = new Temporal.PlainDate(-271821, 5, 18);

const result = d1.until(d2, {
  largestUnit: "year",
  smallestUnit: "day",
  roundingIncrement: 2,
  roundingMode: "expand",
});

TemporalHelpers.assertDuration(result, 0, 0, 0, -2, 0, 0, 0, 0, 0, 0,
  "expand rounding of -1 day to increment 2 near minimum date gives -2 days");
