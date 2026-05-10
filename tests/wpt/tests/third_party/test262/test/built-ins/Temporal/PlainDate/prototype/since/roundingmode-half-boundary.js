// Copyright (C) 2026 Rudolph Gottesheim. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.since
description: Half rounding modes at the exact 0.5 boundary for years and months
info: |
  Tests that all rounding modes correctly break ties at the exact 0.5 boundary
  in RoundRelativeDuration in the negative direction, for both odd and even
  integer parts (distinguishing halfEven from other modes). In the negative
  direction, halfExpand and halfCeil diverge (away from zero vs toward positive
  infinity).

  Years: dates 2019-01-01 / 2020-07-02 produce a difference of -1 year minus
  183 days. The year 2020 is a leap year (366 days), so fractional progress =
  183/366 = 0.5. Dates 2018-01-01 / 2020-07-02 produce -2 years minus 183 days
  (even integer part).

  Months: dates 2019-01-01 / 2019-02-15 produce -1 month minus 14 days.
  February 2019 has 28 days, so fractional progress = 14/28 = 0.5.
  Dates 2018-12-01 / 2019-02-15 produce -2 months minus 14 days (even integer
  part).
includes: [temporalHelpers.js]
features: [Temporal]
---*/

// --- years ---

// -1.5 years: odd integer part (1) + exact 0.5 fractional progress
const yearEarlier1 = new Temporal.PlainDate(2019, 1, 1);
const yearLater = new Temporal.PlainDate(2020, 7, 2);

assert.sameValue(
  yearEarlier1.until(yearLater).total({ unit: "years", relativeTo: yearEarlier1 }),
  1.5,
  "1.5-year duration is on a 0.5 boundary"
);

for (const mode of ["trunc", "ceil", "halfTrunc", "halfCeil"]) {
  TemporalHelpers.assertDuration(
    yearEarlier1.since(yearLater, { smallestUnit: "years", roundingMode: mode }),
    -1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    `-1.5 years with ${mode} rounds toward zero`
  );
}
for (const mode of ["floor", "expand", "halfExpand", "halfFloor", "halfEven"]) {
  TemporalHelpers.assertDuration(
    yearEarlier1.since(yearLater, { smallestUnit: "years", roundingMode: mode }),
    -2, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    `-1.5 years with ${mode} rounds away from zero`
  );
}

// -2.5 years: even integer part (2) — distinguishes halfEven from halfExpand
const yearEarlier2 = new Temporal.PlainDate(2018, 1, 1);

assert.sameValue(
  yearEarlier2.until(yearLater).total({ unit: "years", relativeTo: yearEarlier2 }),
  2.5,
  "2.5-year duration is on a 0.5 boundary"
);

for (const mode of ["trunc", "ceil", "halfTrunc", "halfCeil", "halfEven"]) {
  TemporalHelpers.assertDuration(
    yearEarlier2.since(yearLater, { smallestUnit: "years", roundingMode: mode }),
    -2, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    `-2.5 years with ${mode} rounds toward zero`
  );
}
for (const mode of ["floor", "expand", "halfExpand", "halfFloor"]) {
  TemporalHelpers.assertDuration(
    yearEarlier2.since(yearLater, { smallestUnit: "years", roundingMode: mode }),
    -3, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    `-2.5 years with ${mode} rounds away from zero`
  );
}

// --- months ---

// -1.5 months: odd integer part (1) + exact 0.5 fractional progress
const monthEarlier1 = new Temporal.PlainDate(2019, 1, 1);
const monthLater = new Temporal.PlainDate(2019, 2, 15);

assert.sameValue(
  monthEarlier1.until(monthLater).total({ unit: "months", relativeTo: monthEarlier1 }),
  1.5,
  "1.5-month duration is on a 0.5 boundary"
);

for (const mode of ["trunc", "ceil", "halfTrunc", "halfCeil"]) {
  TemporalHelpers.assertDuration(
    monthEarlier1.since(monthLater, { smallestUnit: "months", roundingMode: mode }),
    0, -1, 0, 0, 0, 0, 0, 0, 0, 0,
    `-1.5 months with ${mode} rounds toward zero`
  );
}
for (const mode of ["floor", "expand", "halfExpand", "halfFloor", "halfEven"]) {
  TemporalHelpers.assertDuration(
    monthEarlier1.since(monthLater, { smallestUnit: "months", roundingMode: mode }),
    0, -2, 0, 0, 0, 0, 0, 0, 0, 0,
    `-1.5 months with ${mode} rounds away from zero`
  );
}

// -2.5 months: even integer part (2) — distinguishes halfEven from halfExpand
const monthEarlier2 = new Temporal.PlainDate(2018, 12, 1);

assert.sameValue(
  monthEarlier2.until(monthLater).total({ unit: "months", relativeTo: monthEarlier2 }),
  2.5,
  "2.5-month duration is on a 0.5 boundary"
);

for (const mode of ["trunc", "ceil", "halfTrunc", "halfCeil", "halfEven"]) {
  TemporalHelpers.assertDuration(
    monthEarlier2.since(monthLater, { smallestUnit: "months", roundingMode: mode }),
    0, -2, 0, 0, 0, 0, 0, 0, 0, 0,
    `-2.5 months with ${mode} rounds toward zero`
  );
}
for (const mode of ["floor", "expand", "halfExpand", "halfFloor"]) {
  TemporalHelpers.assertDuration(
    monthEarlier2.since(monthLater, { smallestUnit: "months", roundingMode: mode }),
    0, -3, 0, 0, 0, 0, 0, 0, 0, 0,
    `-2.5 months with ${mode} rounds away from zero`
  );
}
