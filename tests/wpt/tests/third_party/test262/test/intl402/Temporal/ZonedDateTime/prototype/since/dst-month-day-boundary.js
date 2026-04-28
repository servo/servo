// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.since
description: >
  Difference with the endpoint being the end of a skipped hour, chooses the
  smaller of two possible durations
includes: [temporalHelpers.js]
features: [Temporal]
---*/

// Based on a test case by Adam Shaw

const d1 = new Temporal.ZonedDateTime(957258000_000_000_000n /* = 2000-05-02T02:00-07:00 */, "America/Vancouver");
const d2 = new Temporal.ZonedDateTime(954669600_000_000_000n /* = 2000-04-02T03:00-07:00 */, "America/Vancouver");
// NOTE: nonexistent hour just before d2

const result = d1.since(d2, { largestUnit: "months" });

TemporalHelpers.assertDuration(
  result, 0, 0, 0, 29, 23, 0, 0, 0, 0, 0,
  "Result should not balance up to months, but pick the smaller of two possible durations"
);
