// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Difference across leap months in chinese calendar
esid: sec-temporal.plaindatetime.prototype.until
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

// 2001 is a leap year with a M04L leap month.

const calendar = "chinese";
const options = { overflow: "reject" };

const common1Month4 = Temporal.PlainDateTime.from({ year: 2000, monthCode: "M04", day: 1, hour: 12, minute: 34, calendar }, options);
const common1Month5 = Temporal.PlainDateTime.from({ year: 2000, monthCode: "M05", day: 1, hour: 12, minute: 34, calendar }, options);
const common1Month6 = Temporal.PlainDateTime.from({ year: 2000, monthCode: "M06", day: 1, hour: 12, minute: 34, calendar }, options);
const leapMonth4 = Temporal.PlainDateTime.from({ year: 2001, monthCode: "M04", day: 1, hour: 12, minute: 34, calendar }, options);
const leapMonth4L = Temporal.PlainDateTime.from({ year: 2001, monthCode: "M04L", day: 1, hour: 12, minute: 34, calendar }, options);
const leapMonth5 = Temporal.PlainDateTime.from({ year: 2001, monthCode: "M05", day: 1, hour: 12, minute: 34, calendar }, options);
const common2Month4 = Temporal.PlainDateTime.from({ year: 2002, monthCode: "M04", day: 1, hour: 12, minute: 34, calendar }, options);
const common2Month5 = Temporal.PlainDateTime.from({ year: 2002, monthCode: "M05", day: 1, hour: 12, minute: 34, calendar }, options);

// (receiver, argument, years test data, months test data)
// test data: expected years, months, weeks, days, description
// largestUnit years: make sure some cases where the answer is 12 months do not
// balance up to 1 year
// largestUnit months: similar to years, but make sure number of months in year
// is computed correctly
// For largestUnit of days and weeks, the results should be identical to what
// the ISO calendar gives for the corresponding ISO dates
const tests = [
  [
    common1Month4, leapMonth4,
    [1, 0, 0, 0, "M04-M04 common-leap is 1y"],
    [0, 12, 0, 0, "M04-M04 common-leap is 12mo"],
  ],
  [
    leapMonth4, common2Month4,
    [1, 0, 0, 0, "M04-M04 leap-common is 1y"],
    [0, 13, 0, 0, "M04-M04 leap-common is 13mo not 12mo"],
  ],
  [
    common1Month4, common2Month4,
    [2, 0, 0, 0, "M04-M04 common-common is 2y"],
    [0, 25, 0, 0, "M04-M04 common-common is 25mo not 24mo"],
  ],
  [
    common1Month5, leapMonth5,
    [1, 0, 0, 0, "M05-M05 common-leap is 1y"],
    [0, 13, 0, 0, "M05-M05 common-leap is 13mo not 12mo"],
  ],
  [
    leapMonth5, common2Month5,
    [1, 0, 0, 0, "M05-M05 leap-common is 1y"],
    [0, 12, 0, 0, "M05-M05 leap-common is 12mo"],
  ],
  [
    common1Month5, common2Month5,
    [2, 0, 0, 0, "M05-M05 common-common is 2y"],
    [0, 25, 0, 0, "M05-M05 common-common is 25mo not 24mo"],
  ],
  [
    common1Month4, leapMonth4L,
    [1, 1, 0, 0, "M04-M04L is 1y 1mo"],
    [0, 13, 0, 0, "M04-M04L is 13mo"],
  ],
  [
    leapMonth4L, common2Month4,
    [0, 12, 0, 0, "M04L-M04 is 12mo not 1y"],
    [0, 12, 0, 0, "M04L-M04 is 12mo"],
  ],
  [
    common1Month5, leapMonth4L,
    [0, 12, 0, 0, "M05-M04L is 12mo not 1y"],
    [0, 12, 0, 0, "M05-M04L is 12mo"],
  ],
  [
    leapMonth4L, common2Month5,
    [1, 1, 0, 0, "M04L-M05 is 1y 1mo (exhibits calendar-specific constraining)"],
    [0, 13, 0, 0, "M04L-M05 is 13mo"],
  ],
  [
    common1Month6, leapMonth5,
    [0, 12, 0, 0, "M06-M05 common-leap is 12mo not 11mo"],
    [0, 12, 0, 0, "M06-M05 common-leap is 12mo not 11mo"],
  ],

  // Negative
  [
    common2Month4, leapMonth4,
    [-1, 0, 0, 0, "M04-M04 common-leap backwards is -1y"],
    [0, -13, 0, 0, "M04-M04 common-leap backwards is -13mo not -12mo"],
  ],
  [
    leapMonth4, common1Month4,
    [-1, 0, 0, 0, "M04-M04 leap-common backwards is -1y"],
    [0, -12, 0, 0, "M04-M04 leap-common backwards is -12mo not -13mo"],
  ],
  [
    common2Month4, common1Month4,
    [-2, 0, 0, 0, "M04-M04 common-common backwards is -2y"],
    [0, -25, 0, 0, "M04-M04 common-common backwards is -25mo not -24mo"],
  ],
  [
    common2Month5, leapMonth5,
    [-1, 0, 0, 0, "M05-M05 common-leap backwards is -1y"],
    [0, -12, 0, 0, "M05-M05 common-leap backwards is -12mo not -13mo"],
  ],
  [
    leapMonth5, common1Month5,
    [-1, 0, 0, 0, "M05-M05 leap-common backwards is -1y"],
    [0, -13, 0, 0, "M05-M05 leap-common backwards is -13mo not -12mo"],
  ],
  [
    common2Month5, common1Month5,
    [-2, 0, 0, 0, "M05-M05 common-common backwards is -2y"],
    [0, -25, 0, 0, "M05-M05 common-common backwards is -25mo not -24mo"],
  ],
  [
    common2Month4, leapMonth4L,
    [0, -12, 0, 0, "M04-M04L backwards is -12mo not -1y"],
    [0, -12, 0, 0, "M04-M04L backwards is -12mo"],
  ],
  [
    leapMonth4L, common1Month4,
    [-1, 0, 0, 0, "M04L-M04 backwards is -1y not -1y -1mo (exhibits calendar-specific constraining)"],
    [0, -13, 0, 0, "M04L-M04 backwards is -13mo"],
  ],
  [
    common2Month5, leapMonth4L,
    [-1, -1, 0, 0, "M05-M04L backwards is -1y -1mo"],
    [0, -13, 0, 0, "M05-M04L backwards is -13mo"],
  ],
  [
    leapMonth4L, common1Month5,
    [0, -12, 0, 0, "M04L-M05 backwards is -12mo not -1y"],
    [0, -12, 0, 0, "M04L-M05 backwards is -12mo"],
  ],
];

for (const [one, two, yearsTest, monthsTest] of tests) {
  let [years, months, weeks, days, descr] = yearsTest;
  let result = one.until(two, { largestUnit: "years" });
  TemporalHelpers.assertDuration(result, years, months, weeks, days, 0, 0, 0, 0, 0, 0, descr);

  [years, months, weeks, days, descr] = monthsTest;
  result = one.until(two, { largestUnit: "months" });
  TemporalHelpers.assertDuration(result, years, months, weeks, days, 0, 0, 0, 0, 0, 0, descr);

  const oneISO = one.withCalendar("iso8601");
  const twoISO = two.withCalendar("iso8601");

  const resultWeeks = one.until(two, { largestUnit: "weeks" });
  const resultWeeksISO = oneISO.until(twoISO, { largestUnit: "weeks" });
  TemporalHelpers.assertDurationsEqual(resultWeeks, resultWeeksISO,
    `${one.year}-${one.monthCode}-${one.day} : ${two.year}-${two.monthCode}-${two.day} largestUnit weeks`);

  const resultDays = one.until(two);
  const resultDaysISO = oneISO.until(twoISO);
  TemporalHelpers.assertDurationsEqual(resultDays, resultDaysISO,
    `${one.year}-${one.monthCode}-${one.day} : ${two.year}-${two.monthCode}-${two.day} largestUnit days`);
}
