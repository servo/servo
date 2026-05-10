// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
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

const relativeTo = new Temporal.PlainDate(2025, 6, 14);

{
  const duration = new Temporal.Duration(0, 0, 0, 0, /* hours = */ 14);
  const result = duration.round({
    relativeTo,
    smallestUnit: "hours",
    roundingIncrement: 12,
    roundingMode: "ceil"
  });
  TemporalHelpers.assertDuration(result, 0, 0, 0, 0, 24, 0, 0, 0, 0, 0);
}

{
  const duration = new Temporal.Duration(0, 0, 0, 0, 0, /* minutes = */ 1415);
  const result = duration.round({
    relativeTo,
    smallestUnit: "minutes",
    roundingIncrement: 30,
    roundingMode: "ceil"
  });
  TemporalHelpers.assertDuration(result, 0, 0, 0, 0, 0, 1440, 0, 0, 0, 0);
}

{
  const duration = new Temporal.Duration(0, 0, 0, 0, 0, 0, /* seconds = */ 86375);
  const result = duration.round({
    relativeTo,
    smallestUnit: "seconds",
    roundingIncrement: 30,
    roundingMode: "ceil"
  });
  TemporalHelpers.assertDuration(result, 0, 0, 0, 0, 0, 0, 86400, 0, 0, 0);
}

{
  const duration = new Temporal.Duration(0, 0, 0, 0, 0, 0, 0, /* milliseconds = */ 86399_650);
  const result = duration.round({
    relativeTo,
    smallestUnit: "milliseconds",
    roundingIncrement: 500,
    roundingMode: "ceil"
  });
  TemporalHelpers.assertDuration(result, 0, 0, 0, 0, 0, 0, 0, 86400_000, 0, 0);
}

{
  const duration = new Temporal.Duration(0, 0, 0, 0, 0, 0, 0, 0, /* microseconds = */ 86399_999_650);
  const result = duration.round({
    relativeTo,
    smallestUnit: "microseconds",
    roundingIncrement: 500,
    roundingMode: "ceil"
  });
  TemporalHelpers.assertDuration(result, 0, 0, 0, 0, 0, 0, 0, 0, 86400_000_000, 0);
}

{
  const duration = new Temporal.Duration(0, 0, 0, 0, 0, 0, 0, 0, 0, /* nanoseconds = */ 86399_999_999_650);
  const result = duration.round({
    relativeTo,
    smallestUnit: "nanoseconds",
    roundingIncrement: 500,
    roundingMode: "ceil"
  });
  TemporalHelpers.assertDuration(result, 0, 0, 0, 0, 0, 0, 0, 0, 0, 86400_000_000_000);
}
