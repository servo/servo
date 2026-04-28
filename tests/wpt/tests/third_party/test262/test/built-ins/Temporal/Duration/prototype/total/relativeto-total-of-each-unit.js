// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.total
description: Test representative result for all units, with relativeTo
features: [Temporal]
---*/

const duration = new Temporal.Duration(5, 5, 5, 5, 5, 5, 5, 5, 5, 5);
const plainRelativeTo = new Temporal.PlainDate(2000, 1, 1);
const zonedRelativeTo = new Temporal.ZonedDateTime(63072000_000_000_000n /* = 1972-01-01T00Z */, "UTC");

const dayMilliseconds = 24 * 3600 * 1000;
const fullYears = 5;
const fullMonths = fullYears * 12 + 5 + 1;
const fullDays = 366 + 365 + 365 + 365 + 366 + 31 + 28 + 31 + 30 + 31 + 5 * 7 + 5;
const fullMilliseconds = fullDays * dayMilliseconds + 5 * 3600_000 + 5 * 60_000 + 5000 + 5;
const partialDayMilliseconds = fullMilliseconds - fullDays * dayMilliseconds + 0.005005;
const fractionalDay = partialDayMilliseconds / dayMilliseconds;
const partialYearDays = fullDays - (fullYears * 365 + 2);
const fractionalYear = partialYearDays / 365 + fractionalDay / 365;
const fractionalMonths = (10 /* = 2025-07-11 - 2025-07-01 */ * dayMilliseconds + partialDayMilliseconds) / (31 * dayMilliseconds);
const totalResults = {
  years: fullYears + fractionalYear,
  months: fullMonths + fractionalMonths,
  weeks: Math.floor(fullDays / 7) + (2 + fractionalDay) / 7,
  days: fullDays + fractionalDay,
  hours: fullDays * 24 + partialDayMilliseconds / 3600000,
  minutes: fullDays * 24 * 60 + partialDayMilliseconds / 60000,
  seconds: fullDays * 24 * 60 * 60 + partialDayMilliseconds / 1000,
  milliseconds: fullMilliseconds + 0.005005,
  microseconds: fullMilliseconds * 1000 + 5.005,
  nanoseconds: fullMilliseconds * 1000000 + 5005
};
for (const [unit, expected] of Object.entries(totalResults)) {
  for (const relativeTo of [plainRelativeTo, zonedRelativeTo]) {
    assert.sameValue(
      duration.total({ unit, relativeTo }), expected,
      `Duration.total results for ${unit} relative to ${relativeTo}`
    );
  }
}
