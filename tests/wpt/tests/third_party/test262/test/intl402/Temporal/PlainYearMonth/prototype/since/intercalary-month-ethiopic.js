// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.since
description: >
  Check various basic calculations involving the intercalary month (ethiopic
  calendar)
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "ethiopic";
const options = { overflow: "reject" };

const commonM12 = Temporal.PlainYearMonth.from({ year: 2014, monthCode: "M12", calendar}, options);
const commonIntercalary = Temporal.PlainYearMonth.from({ year: 2014, monthCode: "M13", calendar}, options);
const leapFirst = Temporal.PlainYearMonth.from({ year: 2015, monthCode: "M01", calendar}, options);
const leapM12 = Temporal.PlainYearMonth.from({ year: 2015, monthCode: "M12", calendar}, options);
const leapIntercalary = Temporal.PlainYearMonth.from({ year: 2015, monthCode: "M13", calendar}, options);
const common2First = Temporal.PlainYearMonth.from({ year: 2016, monthCode: "M01", calendar }, options);
const common2Intercalary = Temporal.PlainYearMonth.from({ year: 2016, monthCode: "M13", calendar }, options);

const tests = [
  [
    commonIntercalary, leapIntercalary, "backwards last month of common year to last month of leap year",
    ["years", -1, 0],
    ["months", 0, -13],
  ],
  [
    leapIntercalary, common2Intercalary, "backwards last month of leap year to last month of common year",
    ["years", -1, 0],
    ["months", 0, -13],
  ],
  [
    commonM12, leapFirst, "backwards 2mo passing through intercalary month in common year",
    ["years", 0, -2],
    ["months", 0, -2],
  ],
  [
    leapM12, common2First, "backwards 2mo passing through intercalary month in leap year",
    ["years", 0, -2],
    ["months", 0, -2],
  ],
  [
    common2Intercalary, leapIntercalary, "last month of common year to last month of leap year",
    ["years", 1, 0],
    ["months", 0, 13],
  ],
  [
    leapIntercalary, commonIntercalary, "last month of leap year to last month of common year",
    ["years", 1, 0],
    ["months", 0, 13],
  ],
  [
    leapFirst, commonM12, "2mo passing through intercalary month in common year",
    ["years", 0, 2],
    ["months", 0, 2],
  ],
  [
    common2First, leapM12, "2mo passing through intercalary month in leap year",
    ["years", 0, 2],
    ["months", 0, 2],
  ],
];

for (const [one, two, descr, ...units] of tests) {
  for (const [largestUnit, years, months] of units) {
    TemporalHelpers.assertDuration(
      one.since(two, { largestUnit }),
      years, months, 0, 0, 0, 0, 0, 0, 0, 0,
      descr + ` (largest unit ${largestUnit})`
    );
  }
}
