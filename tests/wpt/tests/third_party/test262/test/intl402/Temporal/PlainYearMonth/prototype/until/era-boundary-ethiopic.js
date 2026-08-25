// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.until
description: Date difference works correctly across era boundaries
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "ethiopic";
const options = { overflow: "reject" };

const aa5500 = Temporal.PlainYearMonth.from({ era: "aa", eraYear: 5500, monthCode: "M01", calendar }, options);
const am1 = Temporal.PlainYearMonth.from({ era: "am", eraYear: 1, monthCode: "M01", calendar }, options);
const am2000 = Temporal.PlainYearMonth.from({ era: "am", eraYear: 2000, monthCode: "M06", calendar }, options);
const am2005 = Temporal.PlainYearMonth.from({ era: "am", eraYear: 2005, monthCode: "M06", calendar }, options);
const aa5450 = Temporal.PlainYearMonth.from({ era: "aa", eraYear: 5450, monthCode: "M07", calendar }, options);
const aa5455 = Temporal.PlainYearMonth.from({ era: "aa", eraYear: 5455, monthCode: "M07", calendar }, options);
const am5 = Temporal.PlainYearMonth.from({ era: "am", eraYear: 5, monthCode: "M01", calendar }, options);

const tests = [
  // From 5500 AA to 1 AM
  [
    aa5500, am1,
    [1, 0, "1y  from 5500 AA to 1 AM"],
    [0, 13, "13mo  from 5500 AA to 1 AM"],
  ],
  [
    am1, aa5500,
    [-1, 0, "-1y backwards  from 5500 AA to 1 AM"],
    [0, -13, "-13mo backwards  from 1 AM to 5500 AA"],
  ],
  // From 2000 AM to 2005 AM
  [
    am2000, am2005,
    [5, 0, "5y from 2000 AM to 2005 AM"],
    [0, 65, "65mo from 2000 AM to 2005 AM"],
  ],
  [
    am2005, am2000,
    [-5, 0, "-5y backwards from 2000 AM to 2005 AM"],
    [0, -65, "-65mo backwards from 2000 AM to 2005 AM"],
  ],
  // From 5450 AA to 5455 AA
  [
    aa5450, aa5455,
    [5, 0, "5y from 5450 AA to 5455 AA"],
    [0, 65, "65mo from 5450 AA to 5455 AA"],
  ],
  [
    aa5455, aa5450,
    [-5, 0, "-5y  backwards from 5450 AA to 5455 AA"],
    [0, -65, "-65mo backwards from 5450 AA to 5455 AA"],
  ],
  // From 5 AM to 5500 AA
  [
    aa5500, am5,
    [5, 0, "5y from 5 AM to 5500 AA"],
    [0, 65, "65mo from 5 AM to 5500 AA"],
  ],
  [
    am5, aa5500,
    [-5, 0, "-5y backwards from 5 AM to 5500 AA"],
    [0, -65, "-65mo backwards from 5 AM to 5500 AA"],
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
