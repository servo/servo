// Copyright (C) 2026 Rudolph Gottesheim. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.since
description: Half rounding modes at the exact 0.5 boundary
info: |
  Tests that all rounding modes correctly break ties at the exact 0.5 boundary
  in RoundRelativeDuration in the negative direction, for both odd and even
  integer parts (distinguishing halfEven from other modes). In the negative
  direction, halfExpand and halfCeil diverge (away from zero vs toward positive
  infinity).

  Calling since() in the negative direction (earlier.since(later)) with
  PlainYearMonth(2018, 6) and PlainYearMonth(2019, 12) produces a difference
  of -1 year minus 6 months. RoundRelativeDuration converts the 6-month
  remainder to days relative to the reference date (1st of the month). From
  2019-06-01, six months spans Jun(30)+Jul(31)+Aug(31)+Sep(30)+Oct(31)+
  Nov(30) = 183 days, and the year from 2019-06-01 to 2020-06-01 contains
  366 days (crossing Feb 29, 2020), giving a fractional progress of exactly
  183/366 = 0.5.

  With PlainYearMonth(2017, 6) and PlainYearMonth(2019, 12) the difference is
  -2 years minus 6 months, giving the same 0.5 fractional progress but with
  an even integer part. This distinguishes halfEven from halfExpand in the
  negative direction.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

// -1.5 years: odd integer part (1) + exact 0.5 fractional progress
const earlier1 = new Temporal.PlainYearMonth(2018, 6);
const later = new Temporal.PlainYearMonth(2019, 12);

assert.sameValue(
  earlier1.until(later).total({ unit: "years", relativeTo: "2018-06-01" }),
  1.5,
  "1.5-year duration is on a 0.5 boundary"
);

for (const mode of ["trunc", "ceil", "halfTrunc", "halfCeil"]) {
  TemporalHelpers.assertDuration(
    earlier1.since(later, { smallestUnit: "years", roundingMode: mode }),
    -1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    `-1.5 years with ${mode} rounds toward zero`
  );
}
for (const mode of ["floor", "expand", "halfExpand", "halfFloor", "halfEven"]) {
  TemporalHelpers.assertDuration(
    earlier1.since(later, { smallestUnit: "years", roundingMode: mode }),
    -2, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    `-1.5 years with ${mode} rounds away from zero`
  );
}

// -2.5 years: even integer part (2) — distinguishes halfEven from halfExpand
const earlier2 = new Temporal.PlainYearMonth(2017, 6);

assert.sameValue(
  earlier2.until(later).total({ unit: "years", relativeTo: "2017-06-01" }),
  2.5,
  "2.5-year duration is on a 0.5 boundary"
);

for (const mode of ["trunc", "ceil", "halfTrunc", "halfCeil", "halfEven"]) {
  TemporalHelpers.assertDuration(
    earlier2.since(later, { smallestUnit: "years", roundingMode: mode }),
    -2, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    `-2.5 years with ${mode} rounds toward zero`
  );
}
for (const mode of ["floor", "expand", "halfExpand", "halfFloor"]) {
  TemporalHelpers.assertDuration(
    earlier2.since(later, { smallestUnit: "years", roundingMode: mode }),
    -3, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    `-2.5 years with ${mode} rounds away from zero`
  );
}
