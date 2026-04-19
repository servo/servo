// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-temporal.zoneddatetime.prototype.since
description: >
    Balancing the resulting duration takes the time zone's UTC offset shifts
    into account
includes: [temporalHelpers.js]
features: [Temporal]
---*/

// Based on a test case by Adam Shaw
{
    const start = new Temporal.ZonedDateTime(
        949132800_000_000_000n /* = 2000-01-29T08Z */,
        "America/Vancouver"); /* = 2000-01-29T00-08 in local time */
    const end = new Temporal.ZonedDateTime(
        972889200_000_000_000n /* = 2000-10-30T07Z */,
        "America/Vancouver"); /* = 2000-10-29T23-08 in local time */

    const duration = start.since(end, { largestUnit: "years" });
    TemporalHelpers.assertDuration(duration, 0, -9, 0, 0, -24, 0, 0, 0, 0, 0,
        "24 hours does not balance to 1 day in 25-hour day");
}
