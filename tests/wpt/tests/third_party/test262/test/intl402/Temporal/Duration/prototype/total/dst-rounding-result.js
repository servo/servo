// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-temporal.duration.prototype.total
description: >
    Rounding the resulting duration takes the time zone's UTC offset shifts
    into account
features: [Temporal]
---*/

// Based on a test case by Adam Shaw

{
  // Date part of duration lands on skipped DST hour, causing disambiguation
  const duration = new Temporal.Duration(0, 1, 0, 15, 11, 30);
  const relativeTo = new Temporal.ZonedDateTime(
    950868000_000_000_000n /* = 2000-02-18T10Z */,
    "America/Vancouver"); /* = 2000-02-18T02-08 in local time */

  assert.sameValue(duration.total({ unit: "months", relativeTo }), 1.5,
    "1 month 15 days 11:30 should be exactly 1.5 months");
}

{
  // Month-only part of duration lands on skipped DST hour, should not cause
  // disambiguation
  const duration = new Temporal.Duration(0, 1, 0, 15, 0, 30);
  const relativeTo = new Temporal.ZonedDateTime(
    951991200_000_000_000n /* = 2000-03-02T10Z */,
    "America/Vancouver"); /* = 2000-03-02T02-08 in local time */

  assert.sameValue(duration.total({ unit: "months", relativeTo }), 1.5,
    "1 month 15 days 00:30 should be exactly 1.5 months");
}
