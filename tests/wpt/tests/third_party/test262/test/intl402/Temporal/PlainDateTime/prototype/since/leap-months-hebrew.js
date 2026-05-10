// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Difference across leap months in hebrew calendar
esid: sec-temporal.plaindatetime.prototype.since
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

// 5784 is a leap year.
// M05 - Shevat
// M05L - Adar I
// M06 - Adar II
// M07 - Nisan

const calendar = "hebrew";
const options = { overflow: "reject" };

const common1Shevat = Temporal.PlainDateTime.from({ year: 5783, monthCode: "M05", day: 1, hour: 12, minute: 34, calendar }, options);
const common1Adar = Temporal.PlainDateTime.from({ year: 5783, monthCode: "M06", day: 1, hour: 12, minute: 34, calendar }, options);
const common1Nisan = Temporal.PlainDateTime.from({ year: 5783, monthCode: "M07", day: 1, hour: 12, minute: 34, calendar }, options);
const leapShevat = Temporal.PlainDateTime.from({ year: 5784, monthCode: "M05", day: 1, hour: 12, minute: 34, calendar }, options);
const leapAdarI = Temporal.PlainDateTime.from({ year: 5784, monthCode: "M05L", day: 1, hour: 12, minute: 34, calendar }, options);
const leapAdarII = Temporal.PlainDateTime.from({ year: 5784, monthCode: "M06", day: 1, hour: 12, minute: 34, calendar }, options);
const common2Shevat = Temporal.PlainDateTime.from({ year: 5785, monthCode: "M05", day: 1, hour: 12, minute: 34, calendar }, options);
const common2Adar = Temporal.PlainDateTime.from({ year: 5785, monthCode: "M06", day: 1, hour: 12, minute: 34, calendar }, options);

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
    common1Shevat, leapShevat,
    [-1, 0, 0, 0, "M05-M05 common-leap backwards is -1y"],
    [0, -12, 0, 0, "M05-M05 common-leap backwards is -12mo"],
  ],
  [
    leapShevat, common2Shevat,
    [-1, 0, 0, 0, "M05-M05 leap-common backwards is -1y"],
    [0, -13, 0, 0, "M05-M05 leap-common backwards is -13mo not -12mo"],
  ],
  [
    common1Shevat, common2Shevat,
    [-2, 0, 0, 0, "M05-M05 common-common backwards is -2y"],
    [0, -25, 0, 0, "M05-M05 common-common backwards is -25mo not -24mo"],
  ],
  [
    common1Adar, leapAdarII,
    [-1, 0, 0, 0, "M06-M06 common-leap backwards is -1y"],
    [0, -13, 0, 0, "M06-M06 common-leap backwards is -13mo not -12mo"],
  ],
  [
    leapAdarII, common2Adar,
    [-1, 0, 0, 0, "M06-M06 leap-common backwards is -1y"],
    [0, -12, 0, 0, "M06-M06 leap-common backwards is -12mo"],
  ],
  [
    common1Adar, common2Adar,
    [-2, 0, 0, 0, "M06-M06 common-common backwards is -2y"],
    [0, -25, 0, 0, "M06-M06 common-common backwards is -25mo not -24mo"],
  ],
  [
    common1Shevat, leapAdarI,
    [-1, -1, 0, 0, "M05-M05L backwards is -1y -1mo"],
    [0, -13, 0, 0, "M05-M05L backwards is -13mo"],
  ],
  [
    leapAdarI, common2Shevat,
    [0, -12, 0, 0, "M05L-M05 backwards is -12mo not -1y"],
    [0, -12, 0, 0, "M05L-M05 backwards is -12mo"],
  ],
  [
    common1Adar, leapAdarI,
    [0, -12, 0, 0, "M06-M05L backwards is -12mo not -1y"],
    [0, -12, 0, 0, "M06-M05L backwards is -12mo"],
  ],
  [
    leapAdarI, common2Adar,
    [-1, 0, 0, 0, "M05L-M06 backwards is -1y (exhibits calendar-specific constraining)"],
    [0, -13, 0, 0, "M05L-M06 backwards is -13mo"],
  ],
  [
    common1Nisan, leapAdarII,
    [0, -12, 0, 0, "M07-M06 common-leap backwards is -12mo not -11mo"],
    [0, -12, 0, 0, "M07-M06 common-leap backwards is -12mo not -11mo"],
  ],

  // Negative
  [
    common2Shevat, leapShevat,
    [1, 0, 0, 0, "M05-M05 common-leap is 1y"],
    [0, 13, 0, 0, "M05-M05 common-leap is 13mo not 12mo"],
  ],
  [
    leapShevat, common1Shevat,
    [1, 0, 0, 0, "M05-M05 leap-common is 1y"],
    [0, 12, 0, 0, "M05-M05 leap-common is 12mo not 13mo"],
  ],
  [
    common2Shevat, common1Shevat,
    [2, 0, 0, 0, "M05-M05 common-common is 2y"],
    [0, 25, 0, 0, "M05-M05 common-common is 25mo not 24mo"],
  ],
  [
    common2Adar, leapAdarII,
    [1, 0, 0, 0, "M06-M06 common-leap is 1y"],
    [0, 12, 0, 0, "M06-M06 common-leap is 12mo not 13mo"],
  ],
  [
    leapAdarII, common1Adar,
    [1, 0, 0, 0, "M06-M06 leap-common is 1y"],
    [0, 13, 0, 0, "M06-M06 leap-common is 13mo not 12mo"],
  ],
  [
    common2Adar, common1Adar,
    [2, 0, 0, 0, "M06-M06 common-common is 2y"],
    [0, 25, 0, 0, "M06-M06 common-common is 25mo not 24mo"],
  ],
  [
    common2Shevat, leapAdarI,
    [0, 12, 0, 0, "M05-M05L is 12mo not 1y"],
    [0, 12, 0, 0, "M05-M05L is 12mo"],
  ],
  [
    leapAdarI, common1Shevat,
    [1, 1, 0, 0, "M05L-M05 is 1y 1mo (exhibits calendar-specific constraining)"],
    [0, 13, 0, 0, "M05L-M05 is 13mo"],
  ],
  [
    common2Adar, leapAdarI,
    [1, 1, 0, 0, "M06-M05L is 1y 1mo"],
    [0, 13, 0, 0, "M06-M05L is 13mo"],
  ],
  [
    leapAdarI, common1Adar,
    [0, 12, 0, 0, "M05L-M06 is 12mo not 1y"],
    [0, 12, 0, 0, "M05L-M06 is 12mo"],
  ],
];

for (const [one, two, yearsTest, monthsTest] of tests) {
  let [years, months, weeks, days, descr] = yearsTest;
  let result = one.since(two, { largestUnit: "years" });
  TemporalHelpers.assertDuration(result, years, months, weeks, days, 0, 0, 0, 0, 0, 0, descr);

  [years, months, weeks, days, descr] = monthsTest;
  result = one.since(two, { largestUnit: "months" });
  TemporalHelpers.assertDuration(result, years, months, weeks, days, 0, 0, 0, 0, 0, 0, descr);

  const oneISO = one.withCalendar("iso8601");
  const twoISO = two.withCalendar("iso8601");

  const resultWeeks = one.since(two, { largestUnit: "weeks" });
  const resultWeeksISO = oneISO.since(twoISO, { largestUnit: "weeks" });
  TemporalHelpers.assertDurationsEqual(resultWeeks, resultWeeksISO,
    `${one.year}-${one.monthCode}-${one.day} : ${two.year}-${two.monthCode}-${two.day} largestUnit weeks`);

  const resultDays = one.since(two);
  const resultDaysISO = oneISO.since(twoISO);
  TemporalHelpers.assertDurationsEqual(resultDays, resultDaysISO,
    `${one.year}-${one.monthCode}-${one.day} : ${two.year}-${two.monthCode}-${two.day} largestUnit days`);
}
