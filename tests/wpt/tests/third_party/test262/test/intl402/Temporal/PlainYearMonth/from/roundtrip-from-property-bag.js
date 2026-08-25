// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.from
description: >
  Check that various dates created from a property bag have the expected
  properties
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const options = { overflow: "reject" };

const cases = [
  ["buddhist", 2543, 1, "M01", "be", 2543, "month containing ISO date 2000-01-01"],
  ["chinese", 1999, 11, "M11", undefined, undefined, "month containing ISO date 2000-01-01"],
  ["coptic", 1716, 4, "M04", "am", 1716, "month containing ISO date 2000-01-01"],
  ["dangi", 1999, 11, "M11", undefined, undefined, "month containing ISO date 2000-01-01"],
  ["ethioaa", 7492, 4, "M04", "aa", 7492, "month containing ISO date 2000-01-01"],
  ["ethiopic", 1992, 4, "M04", "am", 1992, "month containing ISO date 2000-01-01"],
  ["gregory", 2000, 1, "M01", "ce", 2000, "month containing ISO date 2000-01-01"],
  ["hebrew", 5760, 4, "M04", "am", 5760, "month containing ISO date 2000-01-01"],
  ["indian", 1921, 10, "M10", "shaka", 1921, "month containing ISO date 2000-01-01"],
  ["islamic-civil", 1420, 9, "M09", "ah", 1420, "month containing ISO date 2000-01-01"],
  ["islamic-tbla", 1420, 9, "M09", "ah", 1420, "month containing ISO date 2000-01-01"],
  ["islamic-umalqura", 1420, 9, "M09", "ah", 1420, "month containing ISO date 2000-01-01"],
  ["japanese", 2000, 1, "M01", "heisei", 12, "month containing ISO date 2000-01-01"],
  ["persian", 1378, 10, "M10", "ap", 1378, "month containing ISO date 2000-01-01"],
  ["roc", 89, 1, "M01", "roc", 89, "month containing ISO date 2000-01-01"],

  ["buddhist", 544, 1, "M01", "be", 544, "month containing ISO date 0001-01-01"],
  // ["chinese", 0, 12, "M11", undefined, undefined, "month containing ISO date 0001-01-01"], // (out of specified range)
  ["coptic", -283, 5, "M05", "am", -283, "month containing ISO date 0001-01-01"],
  // ["dangi", 0, 12, "M11", undefined, undefined, "month containing ISO date 0001-01-01"], // (out of specified range)
  ["ethioaa", 5493, 5, "M05", "aa", 5493, "month containing ISO date 0001-01-01"],
  ["ethiopic", -7, 5, "M05", "aa", 5493, "month containing ISO date 0001-01-01"],
  ["gregory", 1, 1, "M01", "ce", 1, "month containing ISO date 0001-01-01"],
  ["hebrew", 3761, 4, "M04", "am", 3761, "month containing ISO date 0001-01-01"],
  ["indian", -78, 10, "M10", "shaka", -78, "month containing ISO date 0001-01-01"],
  ["islamic-civil", -640, 5, "M05", "bh", 641, "month containing ISO date 0001-01-01"],
  ["islamic-tbla", -640, 5, "M05", "bh", 641, "month containing ISO date 0001-01-01"],
  ["islamic-umalqura", -640, 5, "M05", "bh", 641, "month containing ISO date 0001-01-01"],
  ["japanese", 1, 1, "M01", "ce", 1, "month containing ISO date 0001-01-01"],
  ["persian", -621, 10, "M10", "ap", -621, "month containing ISO date 0001-01-01"],
  ["roc", -1910, 1, "M01", "broc", 1911, "month containing ISO date 0001-01-01"],

  // Additional cases that were moved in from staging tests, or that we add to
  // catch regressions
  ["chinese", 1900, 1, "M01", undefined, undefined, "start of non-approximated range"],
  ["chinese", 2099, 13, "M12", undefined, undefined, "end of non-approximated range"],
  ["dangi", 1900, 1, "M01", undefined, undefined, "start of non-approximated range"],
  ["dangi", 2049, 12, "M12", undefined, undefined, "end of non-approximated range"],
  ["islamic-civil", 1445, 1, "M01", "ah", 1445, "recent year"],
  ["islamic-tbla", 1445, 1, "M01", "ah", 1445, "recent year"],
  ["islamic-umalqura", 1445, 1, "M01", "ah", 1445, "recent year"],
  ["islamic-umalqura", -6823, 1, "M01", "bh", 6824, "https://github.com/unicode-org/icu4x/issues/4914"],
  ["persian", 1395, 1, "M01", "ap", 1395, "leap year 1395"],
  ["persian", 1396, 1, "M01", "ap", 1396, "common year 1396"],
  ["persian", 1397, 1, "M01", "ap", 1397, "common year 1397"],
  ["persian", 1398, 1, "M01", "ap", 1398, "common year 1398"],
  ["persian", 1399, 1, "M01", "ap", 1399, "leap year 1399"],
  ["persian", 1400, 1, "M01", "ap", 1400, "common year 1400"],
];

for (const [calendar, year, month, monthCode, era, eraYear, descr] of cases) {
  const dateFromYearMonth = Temporal.PlainYearMonth.from({ year, month, calendar }, options);
  TemporalHelpers.assertPlainYearMonth(
    dateFromYearMonth,
    year, month, monthCode,
    `${descr} (${calendar}) - created from year and month`,
    era, eraYear, null);

  const dateFromYearMonthCode = Temporal.PlainYearMonth.from({ year, monthCode, calendar }, options);
  TemporalHelpers.assertPlainYearMonth(
    dateFromYearMonthCode,
    year, month, monthCode,
    `${descr} (${calendar}) - created from year and month code`,
    era, eraYear, null);

  if (era === undefined) continue;  // skip era-less calendars

  const dateFromEraMonth = Temporal.PlainYearMonth.from({ era, eraYear, month, calendar }, options);
  TemporalHelpers.assertPlainYearMonth(
    dateFromEraMonth,
    year, month, monthCode,
    `${descr} (${calendar}) - created from era, era year, and month`,
    era, eraYear, null);

  const dateFromEraMonthCode = Temporal.PlainYearMonth.from({ era, eraYear, monthCode, calendar }, options);
  TemporalHelpers.assertPlainYearMonth(
    dateFromEraMonthCode,
    year, month, monthCode,
    `${descr} (${calendar}) - created from era, era year, and month code`,
    era, eraYear, null);
}
