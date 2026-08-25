// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.from
description: >
  Check that various dates created from a property bag have the expected
  properties
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const options = { overflow: "reject" };

const cases = [
  ["buddhist", 2543, 1, "M01", 1, "be", 2543, "ISO date 2000-01-01"],
  ["chinese", 1999, 11, "M11", 25, undefined, undefined, "ISO date 2000-01-01"],
  ["coptic", 1716, 4, "M04", 22, "am", 1716, "ISO date 2000-01-01"],
  ["dangi", 1999, 11, "M11", 25, undefined, undefined, "ISO date 2000-01-01"],
  ["ethioaa", 7492, 4, "M04", 22, "aa", 7492, "ISO date 2000-01-01"],
  ["ethiopic", 1992, 4, "M04", 22, "am", 1992, "ISO date 2000-01-01"],
  ["gregory", 2000, 1, "M01", 1, "ce", 2000, "ISO date 2000-01-01"],
  ["hebrew", 5760, 4, "M04", 23, "am", 5760, "ISO date 2000-01-01"],
  ["indian", 1921, 10, "M10", 11, "shaka", 1921, "ISO date 2000-01-01"],
  ["islamic-civil", 1420, 9, "M09", 24, "ah", 1420, "ISO date 2000-01-01"],
  ["islamic-tbla", 1420, 9, "M09", 25, "ah", 1420, "ISO date 2000-01-01"],
  ["islamic-umalqura", 1420, 9, "M09", 24, "ah", 1420, "ISO date 2000-01-01"],
  ["japanese", 2000, 1, "M01", 1, "heisei", 12, "ISO date 2000-01-01"],
  ["persian", 1378, 10, "M10", 11, "ap", 1378, "ISO date 2000-01-01"],
  ["roc", 89, 1, "M01", 1, "roc", 89, "ISO date 2000-01-01"],

  // ISO date 0001-01-01
  ["buddhist", 544, 1, "M01", 1, "be", 544, "ISO date 0001-01-01"],
  // ["chinese", 0, 12, "M11", 21, undefined, undefined, "ISO date 0001-01-01"], // (out of specified range)
  ["coptic", -283, 5, "M05", 8, "am", -283, "ISO date 0001-01-01"],
  // ["dangi", 0, 12, "M11", 21, undefined, undefined, "ISO date 0001-01-01"], // (out of specified range)
  ["ethioaa", 5493, 5, "M05", 8, "aa", 5493, "ISO date 0001-01-01"],
  ["ethiopic", -7, 5, "M05", 8, "aa", 5493, "ISO date 0001-01-01"],
  ["gregory", 1, 1, "M01", 1, "ce", 1, "ISO date 0001-01-01"],
  ["hebrew", 3761, 4, "M04", 18, "am", 3761, "ISO date 0001-01-01"],
  ["indian", -78, 10, "M10", 11, "shaka", -78, "ISO date 0001-01-01"],
  ["islamic-civil", -640, 5, "M05", 18, "bh", 641, "ISO date 0001-01-01"],
  ["islamic-tbla", -640, 5, "M05", 19, "bh", 641, "ISO date 0001-01-01"],
  ["islamic-umalqura", -640, 5, "M05", 18, "bh", 641, "ISO date 0001-01-01"],
  ["japanese", 1, 1, "M01", 1, "ce", 1, "ISO date 0001-01-01"],
  ["persian", -621, 10, "M10", 11, "ap", -621, "ISO date 0001-01-01"],
  ["roc", -1910, 1, "M01", 1, "broc", 1911, "ISO date 0001-01-01"],

  // Additional cases that were moved in from staging tests, or that we add to
  // catch regressions
  ["chinese", 1899, 12, "M12", 1, undefined, undefined, "start of non-approximated range"],
  ["chinese", 2099, 13, "M12", 21, undefined, undefined, "end of non-approximated range"],
  ["dangi", 1899, 12, "M12", 1, undefined, undefined, "start of non-approximated range"],
  ["dangi", 2049, 12, "M12", 8, undefined, undefined, "end of non-approximated range"],
  ["islamic-civil", 1445, 1, "M01", 1, "ah", 1445, "recent year"],
  ["islamic-tbla", 1445, 1, "M01", 1, "ah", 1445, "recent year"],
  ["islamic-umalqura", 1445, 1, "M01", 1, "ah", 1445, "recent year"],
  ["islamic-umalqura", -6823, 1, "M01", 1, "bh", 6824, "https://github.com/unicode-org/icu4x/issues/4914"],
  ["persian", 1395, 1, "M01", 1, "ap", 1395, "leap year 1395"],
  ["persian", 1396, 1, "M01", 1, "ap", 1396, "common year 1396"],
  ["persian", 1397, 1, "M01", 1, "ap", 1397, "common year 1397"],
  ["persian", 1398, 1, "M01", 1, "ap", 1398, "common year 1398"],
  ["persian", 1399, 1, "M01", 1, "ap", 1399, "leap year 1399"],
  ["persian", 1400, 1, "M01", 1, "ap", 1400, "common year 1400"],
];

for (const [calendar, year, month, monthCode, day, era, eraYear, descr] of cases) {
  const dateFromYearMonth = Temporal.PlainDate.from({ year, month, day, calendar }, options);
  TemporalHelpers.assertPlainDate(
    dateFromYearMonth,
    year, month, monthCode, day,
    `${descr} (${calendar}) - created from year and month`,
    era, eraYear);

  const dateFromYearMonthCode = Temporal.PlainDate.from({ year, monthCode, day, calendar }, options);
  TemporalHelpers.assertPlainDate(
    dateFromYearMonthCode,
    year, month, monthCode, day,
    `${descr} (${calendar}) - created from year and month code`,
    era, eraYear);

  if (era === undefined) continue;  // skip era-less calendars

  const dateFromEraMonth = Temporal.PlainDate.from({ era, eraYear, month, day, calendar }, options);
  TemporalHelpers.assertPlainDate(
    dateFromEraMonth,
    year, month, monthCode, day,
    `${descr} (${calendar}) - created from era, era year, and month`,
    era, eraYear);

  const dateFromEraMonthCode = Temporal.PlainDate.from({ era, eraYear, monthCode, day, calendar }, options);
  TemporalHelpers.assertPlainDate(
    dateFromEraMonthCode,
    year, month, monthCode, day,
    `${descr} (${calendar}) - created from era, era year, and month code`,
    era, eraYear);
}
