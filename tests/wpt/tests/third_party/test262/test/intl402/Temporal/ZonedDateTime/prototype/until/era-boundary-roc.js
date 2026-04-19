// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.until
description: Date difference works correctly across era boundaries
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "roc";
const options = { overflow: "reject" };

const broc5 = Temporal.ZonedDateTime.from({ era: "broc", eraYear: 5, monthCode: "M03", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const broc3 = Temporal.ZonedDateTime.from({ era: "broc", eraYear: 3, monthCode: "M01", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const broc1 = Temporal.ZonedDateTime.from({ era: "broc", eraYear: 1, monthCode: "M06", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const roc1 = Temporal.ZonedDateTime.from({ era: "roc", eraYear: 1, monthCode: "M06", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const roc5 = Temporal.ZonedDateTime.from({ era: "roc", eraYear: 5, monthCode: "M03", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const roc10 = Temporal.ZonedDateTime.from({ era: "roc", eraYear: 10, monthCode: "M01", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);

const tests = [
  // From BROC 5 to ROC 5
  [
    broc5, roc5,
    [9, 0, 0, 0, "9y  from BROC 5 to ROC 5 (no year 0)"],
    [0, 108, 0, 0, "108mo  from BROC 5 to ROC 5 (no year 0)"],
  ],
  [
    roc5, broc5,
    [-9, 0, 0, 0, "-9y backwards  from BROC 5 to ROC 5 (no year 0)"],
    [0, -108, 0, 0, "-108mo backwards  from BROC 5 to ROC 5 (no year 0)"],
  ],
  // Era boundary
  [
    broc1, roc1,
    [1, 0, 0, 0, "1y from BROC 1 to ROC 1"],
    [0, 12, 0, 0, "12mo from BROC 1 to ROC 1"],
  ],
  [
    roc1, broc1,
    [-1, 0, 0, 0, "-1y backwards from BROC 1 to ROC 1"],
    [0, -12, 0, 0, "-12mo backwards from BROC 1 to ROC 1"],
  ],
  [
    broc3, roc10,
    [12, 0, 0, 0, "12y from BROC 3 to ROC 10"],
    [0, 144, 0, 0, "144mo from BROC 3 to ROC 10"],
  ],
  [
    roc10, broc3,
    [-12, 0, 0, 0, "-12y backwards from BROC 3 to ROC 10"],
    [0, -144, 0, 0, "-144mo backwards from BROC 3 to ROC 10"],
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
