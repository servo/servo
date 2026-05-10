// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.until
description: >
  Check various basic calculations involving the intercalary month (coptic
  calendar)
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "coptic";
const options = { overflow: "reject" };

const commonM12 = Temporal.PlainYearMonth.from({ year: 1738, monthCode: "M12", calendar}, options);
const commonIntercalary = Temporal.PlainYearMonth.from({ year: 1738, monthCode: "M13", calendar}, options);
const leapFirst = Temporal.PlainYearMonth.from({ year: 1739, monthCode: "M01", calendar}, options);
const leapM12 = Temporal.PlainYearMonth.from({ year: 1739, monthCode: "M12", calendar}, options);
const leapIntercalary = Temporal.PlainYearMonth.from({ year: 1739, monthCode: "M13", calendar}, options);
const common2First = Temporal.PlainYearMonth.from({ year: 1740, monthCode: "M01", calendar }, options);
const common2Intercalary = Temporal.PlainYearMonth.from({ year: 1740, monthCode: "M13", calendar }, options);

const tests = [
  [
    commonIntercalary, leapIntercalary, "last month of common year to last month of leap year",
    ["years", 1, 0],
    ["months", 0, 13],
  ],
  [
    leapIntercalary, common2Intercalary, "last month of leap year to last month of common year",
    ["years", 1, 0],
    ["months", 0, 13],
  ],
  [
    commonM12, leapFirst, "2mo passing through intercalary month in common year",
    ["years", 0, 2],
    ["months", 0, 2],
  ],
  [
    leapM12, common2First, "2mo passing through intercalary month in leap year",
    ["years", 0, 2],
    ["months", 0, 2],
  ],
  [
    common2Intercalary, leapIntercalary, "backwards last month of common year to last month of leap year",
    ["years", -1, 0],
    ["months", 0, -13],
  ],
  [
    leapIntercalary, commonIntercalary, "backwards last month of leap year to last month of common year",
    ["years", -1, 0],
    ["months", 0, -13],
  ],
  [
    leapFirst, commonM12, "backwards 2mo passing through intercalary month in common year",
    ["years", 0, -2],
    ["months", 0, -2],
  ],
  [
    common2First, leapM12, "backwards 2mo passing through intercalary month in leap year",
    ["years", 0, -2],
    ["months", 0, -2],
  ],
];

for (const [one, two, descr, ...units] of tests) {
  for (const [largestUnit, years, months] of units) {
    TemporalHelpers.assertDuration(
      one.until(two, { largestUnit }),
      years, months, 0, 0, 0, 0, 0, 0, 0, 0,
      descr + ` (largest unit ${largestUnit})`
    );
  }
}
