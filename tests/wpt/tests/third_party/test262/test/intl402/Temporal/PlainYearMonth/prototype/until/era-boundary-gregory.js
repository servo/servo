// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.until
description: Date difference works correctly across era boundaries
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "gregory";
const options = { overflow: "reject" };

const bce5 = Temporal.PlainYearMonth.from({ era: "bce", eraYear: 5, monthCode: "M06", calendar }, options);
const bce2 = Temporal.PlainYearMonth.from({ era: "bce", eraYear: 2, monthCode: "M12", calendar }, options);
const bce1 = Temporal.PlainYearMonth.from({ era: "bce", eraYear: 1, monthCode: "M06", calendar }, options);
const ce1 = Temporal.PlainYearMonth.from({ era: "ce", eraYear: 1, monthCode: "M06", calendar }, options);
const ce2 = Temporal.PlainYearMonth.from({ era: "ce", eraYear: 2, monthCode: "M01", calendar }, options);
const ce5 = Temporal.PlainYearMonth.from({ era: "ce", eraYear: 5, monthCode: "M06", calendar }, options);

const tests = [
  // From 5 BCE to 5 CE
  [
    bce5, ce5,
    [9, 0, "9y  from 5 BCE to 5 CE (no year 0)"],
    [0, 108, "108mo  from 5 BCE to 5 CE (no year 0)"],
  ],
  [
    ce5, bce5,
    [-9, 0, "-9y backwards  from 5 BCE to 5 CE (no year 0)"],
    [0, -108, "-108mo backwards  from 5 BCE to 5 CE (no year 0)"],
  ],
  // CE-BCE boundary
  [
    bce1, ce1,
    [1, 0, "1y from 1 BCE to 1 CE"],
    [0, 12, "12mo from 1 BCE to 1 CE"],
  ],
  [
    ce1, bce1,
    [-1, 0, "-1y backwards from 1 BCE to 1 CE"],
    [0, -12, "-12mo backwards from 1 BCE to 1 CE"],
  ],
  [
    bce2, ce2,
    [2, 1, "2y 1mo from 2 BCE Dec to 2 CE Jan"],
    [0, 25, "25mo from 2 BCE Dec to 2 CE Jan"],
  ],
  [
    ce2, bce2,
    [-2, -1, "-2y -1mo backwards from 2 BCE Dec to 2 CE Jan"],
    [0, -25, "-25mo backwards from 2 BCE Dec to 2 CE Jan"],
  ],
];

for (const [one, two, yearsTest, monthsTest] of tests) {
  let [years, months, descr] = yearsTest;
  let result = one.until(two, { largestUnit: "years" });
  TemporalHelpers.assertDuration(result, years, months, 0, 0, 0, 0, 0, 0, 0, 0, descr);

  [years, months, descr] = monthsTest;
  result = one.until(two, { largestUnit: "months" });
  TemporalHelpers.assertDuration(result, years, months, 0, 0, 0, 0, 0, 0, 0, 0, descr);
}
