// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.since
description: Rounding can cross unit boundaries up to largestUnit
includes: [temporalHelpers.js]
features: [Temporal]
---*/

// Date units
{
  const earlier = new Temporal.PlainDateTime(2022, 1, 1);
  const later = new Temporal.PlainDateTime(2023, 12, 25);
  const duration = earlier.since(later, { largestUnit: "years", smallestUnit: "months", roundingMode: "expand" });
  TemporalHelpers.assertDuration(duration, -2, 0, 0, 0, 0, 0, 0, 0, 0, 0, "-1 year -11 months balances to -2 years");
}

// Time units
{
  const earlier = new Temporal.PlainDateTime(2000, 5, 2);
  const later = new Temporal.PlainDateTime(2000, 5, 2, 1, 59, 59);
  const duration = earlier.since(later, { largestUnit: "hours", smallestUnit: "minutes", roundingMode: "expand" });
  TemporalHelpers.assertDuration(duration, 0, 0, 0, 0, -2, 0, 0, 0, 0, 0, "-1:59 balances to -2 hours");
}

// Both
{
  const earlier = new Temporal.PlainDateTime(1970, 1, 1);
  const later = new Temporal.PlainDateTime(1971, 12, 31, 23, 59, 59, 999, 999, 999);
  const duration = earlier.since(later, { largestUnit: "years", smallestUnit: "microseconds", roundingMode: "expand" });
  TemporalHelpers.assertDuration(duration, -2, 0, 0, 0, 0, 0, 0, 0, 0, 0, "rounding down 1 ns balances to -2 years");
}
