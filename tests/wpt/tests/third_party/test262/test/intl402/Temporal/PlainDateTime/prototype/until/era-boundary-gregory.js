// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.until
description: Date difference works correctly across era boundaries
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "gregory";
const options = { overflow: "reject" };

const bce5 = Temporal.PlainDateTime.from({ era: "bce", eraYear: 5, monthCode: "M06", day: 15, hour: 12, minute: 34, calendar }, options);
const bce2 = Temporal.PlainDateTime.from({ era: "bce", eraYear: 2, monthCode: "M12", day: 1, hour: 12, minute: 34, calendar }, options);
const bce1 = Temporal.PlainDateTime.from({ era: "bce", eraYear: 1, monthCode: "M06", day: 15, hour: 12, minute: 34, calendar }, options);
const ce1 = Temporal.PlainDateTime.from({ era: "ce", eraYear: 1, monthCode: "M06", day: 15, hour: 12, minute: 34, calendar }, options);
const ce2 = Temporal.PlainDateTime.from({ era: "ce", eraYear: 2, monthCode: "M01", day: 1, hour: 12, minute: 34, calendar }, options);
const ce5 = Temporal.PlainDateTime.from({ era: "ce", eraYear: 5, monthCode: "M06", day: 15, hour: 12, minute: 34, calendar }, options);

const tests = [
  // From 5 BCE to 5 CE
  [
    bce5, ce5,
    [9, 0, 0, 0, "9y  from 5 BCE to 5 CE (no year 0)"],
    [0, 108, 0, 0, "108mo  from 5 BCE to 5 CE (no year 0)"],
  ],
  [
    ce5, bce5,
    [-9, 0, 0, 0, "-9y backwards  from 5 BCE to 5 CE (no year 0)"],
    [0, -108, 0, 0, "-108mo backwards  from 5 BCE to 5 CE (no year 0)"],
  ],
  // CE-BCE boundary
  [
    bce1, ce1,
    [1, 0, 0, 0, "1y from 1 BCE to 1 CE"],
    [0, 12, 0, 0, "12mo from 1 BCE to 1 CE"],
  ],
  [
    ce1, bce1,
    [-1, 0, 0, 0, "-1y backwards from 1 BCE to 1 CE"],
    [0, -12, 0, 0, "-12mo backwards from 1 BCE to 1 CE"],
  ],
  [
    bce2, ce2,
    [2, 1, 0, 0, "2y 1mo from 2 BCE Dec to 2 CE Jan"],
    [0, 25, 0, 0, "25mo from 2 BCE Dec to 2 CE Jan"],
  ],
  [
    ce2, bce2,
    [-2, -1, 0, 0, "-2y -1mo backwards from 2 BCE Dec to 2 CE Jan"],
    [0, -25, 0, 0, "-25mo backwards from 2 BCE Dec to 2 CE Jan"],
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
