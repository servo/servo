// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.until
description: Date difference works correctly across era boundaries
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "islamic-umalqura";
const options = { overflow: "reject" };

const bh5 = Temporal.PlainYearMonth.from({ era: "bh", eraYear: 5, monthCode: "M06", calendar }, options);
const bh2 = Temporal.PlainYearMonth.from({ era: "bh", eraYear: 2, monthCode: "M12", calendar }, options);
const bh1 = Temporal.PlainYearMonth.from({ era: "bh", eraYear: 1, monthCode: "M06", calendar }, options);
const ah1 = Temporal.PlainYearMonth.from({ era: "ah", eraYear: 1, monthCode: "M06", calendar }, options);
const ah2 = Temporal.PlainYearMonth.from({ era: "ah", eraYear: 2, monthCode: "M01", calendar }, options);
const ah5 = Temporal.PlainYearMonth.from({ era: "ah", eraYear: 5, monthCode: "M06", calendar }, options);

const tests = [
  // From 5 BH to 5 AH
  [
    bh5, ah5,
    [9, 0, "9y  from 5 BH to 5 AH (no year 0)"],
    [0, 108, "108mo  from 5 BH to 5 AH (no year 0)"],
  ],
  [
    ah5, bh5,
    [-9, 0, "-9y backwards  from 5 BH to 5 AH (no year 0)"],
    [0, -108, "-108mo backwards  from 5 BH to 5 AH (no year 0)"],
  ],
  // AH-BH boundary
  [
    bh1, ah1,
    [1, 0, "1y from 1 BH to 1 AH"],
    [0, 12, "12mo from 1 BH to 1 AH"],
  ],
  [
    ah1, bh1,
    [-1, 0, "-1y backwards from 1 BH to 1 AH"],
    [0, -12, "-12mo backwards from 1 BH to 1 AH"],
  ],
  [
    bh2, ah2,
    [2, 1, "2y 1mo from 2 BH Dec to 2 AH Jan"],
    [0, 25, "25mo from 2 BH Dec to 2 AH Jan"],
  ],
  [
    ah2, bh2,
    [-2, -1, "-2y -1mo backwards from 2 BH Dec to 2 AH Jan"],
    [0, -25, "-25mo backwards from 2 BH Dec to 2 AH Jan"],
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
