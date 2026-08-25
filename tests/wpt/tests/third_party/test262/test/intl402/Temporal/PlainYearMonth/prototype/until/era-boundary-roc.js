// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.until
description: Date difference works correctly across era boundaries
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "roc";
const options = { overflow: "reject" };

const broc5 = Temporal.PlainYearMonth.from({ era: "broc", eraYear: 5, monthCode: "M03", calendar }, options);
const broc3 = Temporal.PlainYearMonth.from({ era: "broc", eraYear: 3, monthCode: "M01", calendar }, options);
const broc1 = Temporal.PlainYearMonth.from({ era: "broc", eraYear: 1, monthCode: "M06", calendar }, options);
const roc1 = Temporal.PlainYearMonth.from({ era: "roc", eraYear: 1, monthCode: "M06", calendar }, options);
const roc5 = Temporal.PlainYearMonth.from({ era: "roc", eraYear: 5, monthCode: "M03", calendar }, options);
const roc10 = Temporal.PlainYearMonth.from({ era: "roc", eraYear: 10, monthCode: "M01", calendar }, options);

const tests = [
  // From BROC 5 to ROC 5
  [
    broc5, roc5,
    [9, 0, "9y  from BROC 5 to ROC 5 (no year 0)"],
    [0, 108, "108mo  from BROC 5 to ROC 5 (no year 0)"],
  ],
  [
    roc5, broc5,
    [-9, 0, "-9y backwards  from BROC 5 to ROC 5 (no year 0)"],
    [0, -108, "-108mo backwards  from BROC 5 to ROC 5 (no year 0)"],
  ],
  // Era boundary
  [
    broc1, roc1,
    [1, 0, "1y from BROC 1 to ROC 1"],
    [0, 12, "12mo from BROC 1 to ROC 1"],
  ],
  [
    roc1, broc1,
    [-1, 0, "-1y backwards from BROC 1 to ROC 1"],
    [0, -12, "-12mo backwards from BROC 1 to ROC 1"],
  ],
  [
    broc3, roc10,
    [12, 0, "12y from BROC 3 to ROC 10"],
    [0, 144, "144mo from BROC 3 to ROC 10"],
  ],
  [
    roc10, broc3,
    [-12, 0, "-12y backwards from BROC 3 to ROC 10"],
    [0, -144, "-144mo backwards from BROC 3 to ROC 10"],
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
