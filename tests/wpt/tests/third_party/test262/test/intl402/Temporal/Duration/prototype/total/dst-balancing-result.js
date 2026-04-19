// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.total
description: >
    Balancing the resulting duration takes the time zone's UTC offset shifts
    into account
features: [Temporal]
---*/

const oneDay = new Temporal.Duration(0, 0, 0, 1);
const hours25 = new Temporal.Duration(0, 0, 0, 0, 25);

// Samoa skipped 24 hours
let relativeTo = Temporal.PlainDateTime.from("2011-12-29T12:00").toZonedDateTime("Pacific/Apia");
const totalDays = hours25.total({
  unit: "days",
  relativeTo
});

assert(Math.abs(totalDays - (2 + 1 / 24)) < Number.EPSILON,
  "Total days 25 hours over a skipped day");

assert.sameValue(Temporal.Duration.from({ hours: 48 }).total({
  unit: "days",
  relativeTo
}), 3,
  "Total days 48 hours over a skipped day");

assert.sameValue(Temporal.Duration.from({ days: 2 }).total({
  unit: "hours",
  relativeTo
}), 24,
 "Total hours 2 days over a skipped day");

assert.sameValue(Temporal.Duration.from({ days: 3 }).total({
  unit: "hours",
  relativeTo
}), 48,
  "Total hours 3 days over a skipped day");

assert.sameValue(oneDay.total({
  unit: "hours",
  relativeTo: {
    year: 2000,
    month: 10,
    day: 29,
    timeZone: "America/Vancouver"
  }
}), 25,
  "Total hours one day over a repeated hour");

// Based on a test case by Adam Shaw

const duration = new Temporal.Duration(1, 0, 0, 0, 24);
relativeTo = new Temporal.ZonedDateTime(
    941184000_000_000_000n /* = 1999-10-29T08Z */,
    "America/Vancouver"); /* = 1999-10-29T00-08 in local time */

const result = duration.total({ unit: "days", relativeTo });
assert.sameValue(result, 366.96, "24 hours does not balance to 1 day in 25-hour day");
