// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.until
description: >
  Check various basic calculations involving the intercalary month (ethioaa
  calendar)
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "ethioaa";
const options = { overflow: "reject" };

const commonM12 = Temporal.PlainDateTime.from({ year: 7514, monthCode: "M12", day: 1, hour: 12, minute: 34, calendar}, options);
const commonLast = Temporal.PlainDateTime.from({ year: 7514, monthCode: "M13", day: 5, hour: 12, minute: 34, calendar}, options);
const leapFirst = Temporal.PlainDateTime.from({ year: 7515, monthCode: "M01", day: 1, hour: 12, minute: 34, calendar}, options);
const leapM12 = Temporal.PlainDateTime.from({ year: 7515, monthCode: "M12", day: 1, hour: 12, minute: 34, calendar}, options);
const leapPenultimate = Temporal.PlainDateTime.from({ year: 7515, monthCode: "M13", day: 5, hour: 12, minute: 34, calendar}, options);
const leapLast = Temporal.PlainDateTime.from({ year: 7515, monthCode: "M13", day: 6, hour: 12, minute: 34, calendar}, options);
const common2First = Temporal.PlainDateTime.from({ year: 7516, monthCode: "M01", day: 1, hour: 12, minute: 34, calendar }, options);
const common2Last = Temporal.PlainDateTime.from({ year: 7516, monthCode: "M13", day: 5, hour: 12, minute: 34, calendar }, options);

const tests = [
  [
    commonLast, leapLast, "last day of common year to last day of leap year",
    ["years", 1, 0, 0, 1],
    ["months", 0, 13, 0, 1],
    ["weeks", 0, 0, 52, 2],
    ["days", 0, 0, 0, 366],
  ],
  [
    commonLast, leapPenultimate, "last day of common year to penultimate day of leap year",
    ["years", 1, 0, 0, 0],
    ["months", 0, 13, 0, 0],
    ["weeks", 0, 0, 52, 1],
    ["days", 0, 0, 0, 365],
  ],
  [
    leapLast, common2Last, "last day of leap year to last day of common year",
    ["years", 0, 12, 0, 29],
    ["months", 0, 12, 0, 29],
    ["weeks", 0, 0, 52, 1],
    ["days", 0, 0, 0, 365],
  ],
  [
    commonM12, leapFirst, "2mo passing through intercalary month in common year",
    ["years", 0, 2, 0, 0],
    ["months", 0, 2, 0, 0],
    ["weeks", 0, 0, 5, 0],
    ["days", 0, 0, 0, 35],
  ],
  [
    leapM12, common2First, "2mo passing through intercalary month in leap year",
    ["years", 0, 2, 0, 0],
    ["months", 0, 2, 0, 0],
    ["weeks", 0, 0, 5, 1],
    ["days", 0, 0, 0, 36],
  ],
  [
    common2Last, leapLast, "backwards last day of common year to last day of leap year",
    ["years", 0, -12, 0, -5],
    ["months", 0, -12, 0, -5],
    ["weeks", 0, 0, -52, -1],
    ["days", 0, 0, 0, -365],
  ],
  [
    common2Last, leapPenultimate, "backwards last day of common year to penultimate day of leap year",
    ["years", -1, 0, 0, 0],
    ["months", 0, -13, 0, 0],
    ["weeks", 0, 0, -52, -2],
    ["days", 0, 0, 0, -366],
  ],
  [
    // Note this result: in NonISODateUntil, day is constrained after
    // determining number of years and months added, so this is not 1y 1d or
    // 13mo 1d
    leapLast, commonLast, "backwards last day of leap year to last day of common year",
    ["years", -1, 0, 0, 0],
    ["months", 0, -13, 0, 0],
    ["weeks", 0, 0, -52, -2],
    ["days", 0, 0, 0, -366],
  ],
  [
    leapFirst, commonM12, "backwards 2mo passing through intercalary month in common year",
    ["years", 0, -2, 0, 0],
    ["months", 0, -2, 0, 0],
    ["weeks", 0, 0, -5, 0],
    ["days", 0, 0, 0, -35],
  ],
  [
    common2First, leapM12, "backwards 2mo passing through intercalary month in leap year",
    ["years", 0, -2, 0, 0],
    ["months", 0, -2, 0, 0],
    ["weeks", 0, 0, -5, -1],
    ["days", 0, 0, 0, -36],
  ],
];

for (const [one, two, descr, ...units] of tests) {
  for (const [largestUnit, years, months, weeks, days] of units) {
    TemporalHelpers.assertDuration(
      one.until(two, { largestUnit }),
      years, months, weeks, days, 0, 0, 0, 0, 0, 0,
      descr + ` (largest unit ${largestUnit})`
    );
  }
}
