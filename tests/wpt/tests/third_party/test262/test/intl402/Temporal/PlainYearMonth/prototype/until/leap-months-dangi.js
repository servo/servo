// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Difference across leap months in dangi calendar
esid: sec-temporal.plainyearmonth.prototype.until
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

// 2001 is a leap year with a M04L leap month.

const calendar = "dangi";
const options = { overflow: "reject" };

const common1Month4 = Temporal.PlainYearMonth.from({ year: 2000, monthCode: "M04", calendar }, options);
const common1Month5 = Temporal.PlainYearMonth.from({ year: 2000, monthCode: "M05", calendar }, options);
const common1Month6 = Temporal.PlainYearMonth.from({ year: 2000, monthCode: "M06", calendar }, options);
const leapMonth4 = Temporal.PlainYearMonth.from({ year: 2001, monthCode: "M04", calendar }, options);
const leapMonth4L = Temporal.PlainYearMonth.from({ year: 2001, monthCode: "M04L", calendar }, options);
const leapMonth5 = Temporal.PlainYearMonth.from({ year: 2001, monthCode: "M05", calendar }, options);
const common2Month4 = Temporal.PlainYearMonth.from({ year: 2002, monthCode: "M04", calendar }, options);
const common2Month5 = Temporal.PlainYearMonth.from({ year: 2002, monthCode: "M05", calendar }, options);

// (receiver, argument, years test data, months test data)
// test data: expected years, months, description
// largestUnit years: make sure some cases where the answer is 12 months do not
// balance up to 1 year
// largestUnit months: similar to years, but make sure number of months in year
// is computed correctly
const tests = [
  [
    common1Month4, leapMonth4,
    [1, 0, "M04-M04 common-leap is 1y"],
    [0, 12, "M04-M04 common-leap is 12mo"],
  ],
  [
    leapMonth4, common2Month4,
    [1, 0, "M04-M04 leap-common is 1y"],
    [0, 13, "M04-M04 leap-common is 13mo not 12mo"],
  ],
  [
    common1Month4, common2Month4,
    [2, 0, "M04-M04 common-common is 2y"],
    [0, 25, "M04-M04 common-common is 25mo not 24mo"],
  ],
  [
    common1Month5, leapMonth5,
    [1, 0, "M05-M05 common-leap is 1y"],
    [0, 13, "M05-M05 common-leap is 13mo not 12mo"],
  ],
  [
    leapMonth5, common2Month5,
    [1, 0, "M05-M05 leap-common is 1y"],
    [0, 12, "M05-M05 leap-common is 12mo"],
  ],
  [
    common1Month5, common2Month5,
    [2, 0, "M05-M05 common-common is 2y"],
    [0, 25, "M05-M05 common-common is 25mo not 24mo"],
  ],
  [
    common1Month4, leapMonth4L,
    [1, 1, "M04-M04L is 1y 1mo"],
    [0, 13, "M04-M04L is 13mo"],
  ],
  [
    leapMonth4L, common2Month4,
    [0, 12, "M04L-M04 is 12mo not 1y"],
    [0, 12, "M04L-M04 is 12mo"],
  ],
  [
    common1Month5, leapMonth4L,
    [0, 12, "M05-M04L is 12mo not 1y"],
    [0, 12, "M05-M04L is 12mo"],
  ],
  [
    leapMonth4L, common2Month5,
    [1, 1, "M04L-M05 is 1y 1mo (exhibits calendar-specific constraining)"],
    [0, 13, "M04L-M05 is 13mo"],
  ],
  [
    common1Month6, leapMonth5,
    [0, 12, "M06-M05 common-leap is 12mo not 11mo"],
    [0, 12, "M06-M05 common-leap is 12mo not 11mo"],
  ],

  // Negative
  [
    common2Month4, leapMonth4,
    [-1, 0, "M04-M04 common-leap backwards is -1y"],
    [0, -13, "M04-M04 common-leap backwards is -13mo not -12mo"],
  ],
  [
    leapMonth4, common1Month4,
    [-1, 0, "M04-M04 leap-common backwards is -1y"],
    [0, -12, "M04-M04 leap-common backwards is -12mo not -13mo"],
  ],
  [
    common2Month4, common1Month4,
    [-2, 0, "M04-M04 common-common backwards is -2y"],
    [0, -25, "M04-M04 common-common backwards is -25mo not -24mo"],
  ],
  [
    common2Month5, leapMonth5,
    [-1, 0, "M05-M05 common-leap backwards is -1y"],
    [0, -12, "M05-M05 common-leap backwards is -12mo not -13mo"],
  ],
  [
    leapMonth5, common1Month5,
    [-1, 0, "M05-M05 leap-common backwards is -1y"],
    [0, -13, "M05-M05 leap-common backwards is -13mo not -12mo"],
  ],
  [
    common2Month5, common1Month5,
    [-2, 0, "M05-M05 common-common backwards is -2y"],
    [0, -25, "M05-M05 common-common backwards is -25mo not -24mo"],
  ],
  [
    common2Month4, leapMonth4L,
    [0, -12, "M04-M04L backwards is -12mo not -1y"],
    [0, -12, "M04-M04L backwards is -12mo"],
  ],
  [
    leapMonth4L, common1Month4,
    [-1, 0, "M04L-M04 backwards is -1y not -1y -1mo (exhibits calendar-specific constraining)"],
    [0, -13, "M04L-M04 backwards is -13mo"],
  ],
  [
    common2Month5, leapMonth4L,
    [-1, -1, "M05-M04L backwards is -1y -1mo"],
    [0, -13, "M05-M04L backwards is -13mo"],
  ],
  [
    leapMonth4L, common1Month5,
    [0, -12, "M04L-M05 backwards is -12mo not -1y"],
    [0, -12, "M04L-M05 backwards is -12mo"],
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
