// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.until
description: Rounds relative to a date, bubbling up time units.
info: |
  https://github.com/tc39/proposal-temporal/issues/3121
  Test case that calls BubbleRelativeDuration with a time unit as largestUnit.
  This happens when time units round up and overflow a date unit, and
  largestUnit is not "days" or higher, and the rounding is relative to a
  PlainDateTime (ZonedDateTime would use Instant semantics in that case.)
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const earlier = new Temporal.PlainDateTime(2025, 6, 14);

{
  const later = new Temporal.PlainDateTime(2025, 6, 14, 14);
  const result = earlier.until(later, {
    largestUnit: "hours",
    smallestUnit: "hours",
    roundingIncrement: 12,
    roundingMode: "ceil"
  });
  TemporalHelpers.assertDuration(result, 0, 0, 0, 0, 24, 0, 0, 0, 0, 0);
}

{
  const later = new Temporal.PlainDateTime(2025, 6, 14, 23, 35);
  const result = earlier.until(later, {
    largestUnit: "minutes",
    smallestUnit: "minutes",
    roundingIncrement: 30,
    roundingMode: "ceil"
  });
  TemporalHelpers.assertDuration(result, 0, 0, 0, 0, 0, 1440, 0, 0, 0, 0);
}

{
  const later = new Temporal.PlainDateTime(2025, 6, 14, 23, 59, 35);
  const result = earlier.until(later, {
    largestUnit: "seconds",
    smallestUnit: "seconds",
    roundingIncrement: 30,
    roundingMode: "ceil"
  });
  TemporalHelpers.assertDuration(result, 0, 0, 0, 0, 0, 0, 86400, 0, 0, 0);
}

{
  const later = new Temporal.PlainDateTime(2025, 6, 14, 23, 59, 59, 650);
  const result = earlier.until(later, {
    largestUnit: "milliseconds",
    smallestUnit: "milliseconds",
    roundingIncrement: 500,
    roundingMode: "ceil"
  });
  TemporalHelpers.assertDuration(result, 0, 0, 0, 0, 0, 0, 0, 86400_000, 0, 0);
}

{
  const later = new Temporal.PlainDateTime(2025, 6, 14, 23, 59, 59, 999, 650);
  const result = earlier.until(later, {
    largestUnit: "microseconds",
    smallestUnit: "microseconds",
    roundingIncrement: 500,
    roundingMode: "ceil"
  });
  TemporalHelpers.assertDuration(result, 0, 0, 0, 0, 0, 0, 0, 0, 86400_000_000, 0);
}

{
  const later = new Temporal.PlainDateTime(2025, 6, 14, 23, 59, 59, 999, 999, 650);
  const result = earlier.until(later, {
    largestUnit: "nanoseconds",
    smallestUnit: "nanoseconds",
    roundingIncrement: 500,
    roundingMode: "ceil"
  });
  TemporalHelpers.assertDuration(result, 0, 0, 0, 0, 0, 0, 0, 0, 0, 86400_000_000_000);
}
