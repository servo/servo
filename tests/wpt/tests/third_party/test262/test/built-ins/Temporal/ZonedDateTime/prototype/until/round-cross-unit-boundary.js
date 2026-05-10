// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.until
description: Rounding can cross unit boundaries up to largestUnit
includes: [temporalHelpers.js]
features: [Temporal]
---*/

// Date units
{
  const earlier = new Temporal.ZonedDateTime(1640995200_000_000_000n /* = 2022-01-01T00 */, "UTC");
  const later = new Temporal.ZonedDateTime(1703462400_000_000_000n /* = 2023-12-25T00 */, "UTC");
  const duration = earlier.until(later, { largestUnit: "years", smallestUnit: "months", roundingMode: "expand" });
  TemporalHelpers.assertDuration(duration, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, "1 year 11 months balances to 2 years");
}

// Time units
{
  const earlier = new Temporal.ZonedDateTime(0n, "UTC");
  const later = new Temporal.ZonedDateTime(7199_000_000_000n, "UTC");
  const duration = earlier.until(later, { largestUnit: "hours", smallestUnit: "minutes", roundingMode: "expand" });
  TemporalHelpers.assertDuration(duration, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, "1:59 balances to 2 hours");
}

// Both
{
  const earlier = new Temporal.ZonedDateTime(0n, "UTC");
  const later = new Temporal.ZonedDateTime(63071999_999_999_999n /* = 1971-12-31T23:59:59.999999999 */, "UTC");
  const duration = earlier.until(later, { largestUnit: "years", smallestUnit: "microseconds", roundingMode: "expand" });
  TemporalHelpers.assertDuration(duration, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, "rounding up 1 ns balances to 2 years");
}
