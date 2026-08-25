// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: >
  Rare edge case where presence or absence of relativeTo affects the rounding
  behaviour of rounding mode halfEven
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const plainRelativeTo = new Temporal.PlainDate(1970, 1, 1);
const zonedRelativeTo = new Temporal.ZonedDateTime(0n, "UTC");

const duration = new Temporal.Duration(0, 0, 0, 3, 12);  // 3 days 12 hours
const commonOptions = { smallestUnit: "hours", roundingIncrement: 8, roundingMode: "halfEven" };

// 3 days 12 hours is 10.5 increments, so halfEven rounds down to 10 increments
TemporalHelpers.assertDuration(
  duration.round(commonOptions),
  0, 0, 0, 3, 8, 0, 0, 0, 0, 0, // 3 days 8 hours
  "halfEven rounding is downward with no relativeTo"
);

// Here also we calculate 10.5 increments
TemporalHelpers.assertDuration(
  duration.round({ ...commonOptions, relativeTo: plainRelativeTo }),
  0, 0, 0, 3, 8, 0, 0, 0, 0, 0, // 3 days 8 hours
  "halfEven rounding is downward with PlainDate relativeTo"
);

// Since days can be different lengths when relative to ZonedDateTime, the days
// are accounted separately. 0 days 12 hours is 1.5 increments, so halfEven
// rounds up to 2 increments
TemporalHelpers.assertDuration(
  duration.round({ ...commonOptions, relativeTo: zonedRelativeTo }),
  0, 0, 0, 3, 16, 0, 0, 0, 0, 0, // 3 days 16 hours
  "halfEven rounding is upward with ZonedDateTime relativeTo"
);
