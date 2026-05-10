// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-temporal.zoneddatetime.prototype.since
description: >
    Rounding the resulting duration takes the time zone's UTC offset shifts
    into account
includes: [temporalHelpers.js]
features: [Temporal]
---*/

// Based on a test case by Adam Shaw

{
  // Month-only part of duration lands on skipped DST hour, should not cause
  // disambiguation
  const start = new Temporal.ZonedDateTime(
    950868000_000_000_000n /* = 2000-02-18T10Z */,
    "America/Vancouver"); /* = 2000-02-18T02-08 in local time */
  const end = new Temporal.ZonedDateTime(
    954709200_000_000_000n /* = 2000-04-02T21Z */,
    "America/Vancouver"); /* = 2000-04-02T14-07 in local time */

  const duration = start.since(end, { largestUnit: "months" });
  TemporalHelpers.assertDuration(duration, 0, -1, 0, -15, -11, 0, 0, 0, 0, 0,
    "1-month rounding window is shortened by DST");
}


{
  // Month-only part of duration lands on skipped DST hour, should not cause
  // disambiguation
  const start = new Temporal.ZonedDateTime(
    951991200_000_000_000n /* = 2000-03-02T10Z */,
    "America/Vancouver"); /* = 2000-03-02T02-08 in local time */
  const end = new Temporal.ZonedDateTime(
    956005200_000_000_000n /* = 2000-04-17T21Z */,
    "America/Vancouver"); /* = 2000-04-17T14-07 in local time */

  const duration = start.since(end, { largestUnit: "months" });
  TemporalHelpers.assertDuration(duration, 0, -1, 0, -15, -12, 0, 0, 0, 0, 0,
    "1-month rounding window is not shortened by DST");
}
