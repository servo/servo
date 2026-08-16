// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: Test a specific buggy case from temporal_rs
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const relativeTo = Temporal.ZonedDateTime.from('2025-03-09T03:00:00-07:00[America/Vancouver]');
TemporalHelpers.assertDuration(new Temporal.Duration(0, 1, 0, 30).round({
  smallestUnit: "months",
  roundingIncrement: 2,
  relativeTo,
}), 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, `1m30d to 2m with relativeTo 2025-03-09T03:00:00-07:00[America/Vancouver]`);
