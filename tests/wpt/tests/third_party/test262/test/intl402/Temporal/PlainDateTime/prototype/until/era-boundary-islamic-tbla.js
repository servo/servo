// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.until
description: Date difference works correctly across era boundaries
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "islamic-tbla";
const options = { overflow: "reject" };

const bh5 = Temporal.PlainDateTime.from({ era: "bh", eraYear: 5, monthCode: "M06", day: 15, hour: 12, minute: 34, calendar }, options);
const bh2 = Temporal.PlainDateTime.from({ era: "bh", eraYear: 2, monthCode: "M12", day: 1, hour: 12, minute: 34, calendar }, options);
const bh1 = Temporal.PlainDateTime.from({ era: "bh", eraYear: 1, monthCode: "M06", day: 15, hour: 12, minute: 34, calendar }, options);
const ah1 = Temporal.PlainDateTime.from({ era: "ah", eraYear: 1, monthCode: "M06", day: 15, hour: 12, minute: 34, calendar }, options);
const ah2 = Temporal.PlainDateTime.from({ era: "ah", eraYear: 2, monthCode: "M01", day: 1, hour: 12, minute: 34, calendar }, options);
const ah5 = Temporal.PlainDateTime.from({ era: "ah", eraYear: 5, monthCode: "M06", day: 15, hour: 12, minute: 34, calendar }, options);

const tests = [
  // From 5 BH to 5 AH
  [
    bh5, ah5,
    [9, 0, 0, 0, "9y  from 5 BH to 5 AH (no year 0)"],
    [0, 108, 0, 0, "108mo  from 5 BH to 5 AH (no year 0)"],
  ],
  [
    ah5, bh5,
    [-9, 0, 0, 0, "-9y backwards  from 5 BH to 5 AH (no year 0)"],
    [0, -108, 0, 0, "-108mo backwards  from 5 BH to 5 AH (no year 0)"],
  ],
  // AH-BH boundary
  [
    bh1, ah1,
    [1, 0, 0, 0, "1y from 1 BH to 1 AH"],
    [0, 12, 0, 0, "12mo from 1 BH to 1 AH"],
  ],
  [
    ah1, bh1,
    [-1, 0, 0, 0, "-1y backwards from 1 BH to 1 AH"],
    [0, -12, 0, 0, "-12mo backwards from 1 BH to 1 AH"],
  ],
  [
    bh2, ah2,
    [2, 1, 0, 0, "2y 1mo from 2 BH Dec to 2 AH Jan"],
    [0, 25, 0, 0, "25mo from 2 BH Dec to 2 AH Jan"],
  ],
  [
    ah2, bh2,
    [-2, -1, 0, 0, "-2y -1mo backwards from 2 BH Dec to 2 AH Jan"],
    [0, -25, 0, 0, "-25mo backwards from 2 BH Dec to 2 AH Jan"],
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
