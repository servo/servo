// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-temporal.duration.prototype.round
description: >
    Rounding the resulting duration takes the time zone's UTC offset shifts
    into account
includes: [temporalHelpers.js]
features: [Temporal]
---*/

// Based on a test case by Adam Shaw

{
  // Date part of duration lands on skipped DST hour, causing disambiguation
  const duration = new Temporal.Duration(0, 1, 0, 15, 11, 30);
  const relativeTo = new Temporal.ZonedDateTime(
    950868000_000_000_000n /* = 2000-02-18T10Z */,
    "America/Vancouver"); /* = 2000-02-18T02-08 in local time */

  TemporalHelpers.assertDuration(duration.round({ smallestUnit: "months", relativeTo }),
    0, 2, 0, 0, 0, 0, 0, 0, 0, 0,
    "1 month 15 days 12 hours should be exactly 1.5 months, which rounds up to 2 months");
  TemporalHelpers.assertDuration(duration.round({ smallestUnit: "months", roundingMode: 'halfTrunc', relativeTo }),
    0, 1, 0, 0, 0, 0, 0, 0, 0, 0,
    "1 month 15 days 12 hours should be exactly 1.5 months, which rounds down to 1 month");
}

{
  // Month-only part of duration lands on skipped DST hour
  const duration = new Temporal.Duration(0, 1, 0, 15, 0, 30);
  const relativeTo = new Temporal.ZonedDateTime(
    951991200_000_000_000n /* = 2000-03-02T10Z */,
    "America/Vancouver"); /* = 2000-03-02T02-08 in local time */

  TemporalHelpers.assertDuration(duration.round({ smallestUnit: "months", relativeTo }),
    0, 2, 0, 0, 0, 0, 0, 0, 0, 0,
    "1 month 15 days 00:30 should be exactly 1.5 months, which rounds up to 2 months");
  TemporalHelpers.assertDuration(duration.round({ smallestUnit: "months", roundingMode: 'halfTrunc', relativeTo }),
    0, 1, 0, 0, 0, 0, 0, 0, 0, 0,
    "1 month 15 days 00:30 should be exactly 1.5 months, which rounds down to 1 month");
}

{
  // Day rounding
  // DST spring-forward hour skipped at 2000-04-02T02:00 (23 hour day)
  // 11.5 hours is 0.5
  const duration = new Temporal.Duration(0, 0, 0, 0, 11, 30);
  const relativeTo = new Temporal.PlainDateTime(2000, 4, 2).toZonedDateTime("America/Vancouver");

  TemporalHelpers.assertDuration(
    duration.round({ relativeTo, smallestUnit: "days" }),
    0, 0, 0, 1, 0, 0, 0, 0, 0, 0,
  );

  TemporalHelpers.assertDuration(
    duration.round({ relativeTo, smallestUnit: "days", roundingMode: "halfTrunc" }),
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  );
}
