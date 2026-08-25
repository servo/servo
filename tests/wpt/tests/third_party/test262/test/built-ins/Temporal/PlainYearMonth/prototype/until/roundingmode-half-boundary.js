// Copyright (C) 2026 Rudolph Gottesheim. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.until
description: Half rounding modes at the exact 0.5 boundary
info: |
  Tests that all rounding modes correctly break ties at the exact 0.5 boundary
  in RoundRelativeDuration, for both odd and even integer parts (distinguishing
  halfEven from other modes).

  PlainYearMonth(2018, 6).until(PlainYearMonth(2019, 12)) produces a
  difference of 1 year plus 6 months. RoundRelativeDuration converts the
  6-month remainder to days relative to the reference date (1st of the month).
  From 2019-06-01, six months spans Jun(30)+Jul(31)+Aug(31)+Sep(30)+Oct(31)+
  Nov(30) = 183 days, and the year from 2019-06-01 to 2020-06-01 contains 366
  days (crossing Feb 29, 2020), giving a fractional progress of exactly
  183/366 = 0.5.

  PlainYearMonth(2017, 6).until(PlainYearMonth(2019, 12)) produces 2 years
  plus 6 months with the same 0.5 fractional progress but an even integer
  part. This distinguishes halfEven from halfExpand: halfEven rounds to the
  nearest even integer (2), while halfExpand rounds away from zero (3).
includes: [temporalHelpers.js]
features: [Temporal]
---*/

// 1.5 years: odd integer part (1) + exact 0.5 fractional progress
const earlier1 = new Temporal.PlainYearMonth(2018, 6);
const later = new Temporal.PlainYearMonth(2019, 12);

assert.sameValue(
  earlier1.until(later).total({ unit: "years", relativeTo: "2018-06-01" }),
  1.5,
  "1.5-year duration is on a 0.5 boundary"
);

for (const mode of ["trunc", "floor", "halfTrunc", "halfFloor"]) {
  TemporalHelpers.assertDuration(
    earlier1.until(later, { smallestUnit: "years", roundingMode: mode }),
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    `1.5 years with ${mode} rounds down to 1`
  );
}
for (const mode of ["ceil", "expand", "halfExpand", "halfCeil", "halfEven"]) {
  TemporalHelpers.assertDuration(
    earlier1.until(later, { smallestUnit: "years", roundingMode: mode }),
    2, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    `1.5 years with ${mode} rounds up to 2`
  );
}

// 2.5 years: even integer part (2) — distinguishes halfEven from halfExpand
const earlier2 = new Temporal.PlainYearMonth(2017, 6);

assert.sameValue(
  earlier2.until(later).total({ unit: "years", relativeTo: "2017-06-01" }),
  2.5,
  "2.5-year duration is on a 0.5 boundary"
);

for (const mode of ["trunc", "floor", "halfTrunc", "halfFloor", "halfEven"]) {
  TemporalHelpers.assertDuration(
    earlier2.until(later, { smallestUnit: "years", roundingMode: mode }),
    2, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    `2.5 years with ${mode} rounds down to 2`
  );
}
for (const mode of ["ceil", "expand", "halfExpand", "halfCeil"]) {
  TemporalHelpers.assertDuration(
    earlier2.until(later, { smallestUnit: "years", roundingMode: mode }),
    3, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    `2.5 years with ${mode} rounds up to 3`
  );
}
