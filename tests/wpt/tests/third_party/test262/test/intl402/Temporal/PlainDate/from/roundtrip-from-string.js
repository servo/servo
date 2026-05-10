// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.from
description: >
  Check that various dates created from a RFC 9557 string have the expected
  properties
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const year2000Cases = [
  ["buddhist", 2543, 1, "M01", 1, "be", 2543],
  ["chinese", 1999, 11, "M11", 25, undefined, undefined],
  ["coptic", 1716, 4, "M04", 22, "am", 1716],
  ["dangi", 1999, 11, "M11", 25, undefined, undefined],
  ["ethioaa", 7492, 4, "M04", 22, "aa", 7492],
  ["ethiopic", 1992, 4, "M04", 22, "am", 1992],
  ["gregory", 2000, 1, "M01", 1, "ce", 2000],
  ["hebrew", 5760, 4, "M04", 23, "am", 5760],
  ["indian", 1921, 10, "M10", 11, "shaka", 1921],
  ["islamic-civil", 1420, 9, "M09", 24, "ah", 1420],
  ["islamic-tbla", 1420, 9, "M09", 25, "ah", 1420],
  ["islamic-umalqura", 1420, 9, "M09", 24, "ah", 1420],
  ["japanese", 2000, 1, "M01", 1, "heisei", 12],
  ["persian", 1378, 10, "M10", 11, "ap", 1378],
  ["roc", 89, 1, "M01", 1, "roc", 89],
];

for (const [calendar, year, month, monthCode, day, era, eraYear, descr] of year2000Cases) {
  const string = `2000-01-01[u-ca=${calendar}]`;
  const dateFromString = Temporal.PlainDate.from(string);
  TemporalHelpers.assertPlainDate(
    dateFromString,
    year, month, monthCode, day,
    `${descr} - created from string ${string}`,
    era, eraYear);
}

const year1Cases = [
  ["buddhist", 544, 1, "M01", 1, "be", 544],
  // ["chinese", 0, 12, "M11", 21, undefined, undefined], // (out of specified range)
  ["coptic", -283, 5, "M05", 8, "am", -283],
  // ["dangi", 0, 12, "M11", 21, undefined, undefined], // (out of specified range)
  ["ethioaa", 5493, 5, "M05", 8, "aa", 5493],
  ["ethiopic", -7, 5, "M05", 8, "aa", 5493],
  ["gregory", 1, 1, "M01", 1, "ce", 1],
  ["hebrew", 3761, 4, "M04", 18, "am", 3761],
  ["indian", -78, 10, "M10", 11, "shaka", -78],
  ["islamic-civil", -640, 5, "M05", 18, "bh", 641],
  ["islamic-tbla", -640, 5, "M05", 19, "bh", 641],
  ["islamic-umalqura", -640, 5, "M05", 18, "bh", 641],
  ["japanese", 1, 1, "M01", 1, "ce", 1],
  ["persian", -621, 10, "M10", 11, "ap", -621],
  ["roc", -1910, 1, "M01", 1, "broc", 1911],
];

for (const [calendar, year, month, monthCode, day, era, eraYear, descr] of year1Cases) {
  const string = `0001-01-01[u-ca=${calendar}]`;
  const dateFromString = Temporal.PlainDate.from(string);
  TemporalHelpers.assertPlainDate(
    dateFromString,
    year, month, monthCode, day,
    `${descr} - created from string ${string}`,
    era, eraYear);
}

// Additional cases that were moved in from staging tests, or that we add to
// catch regressions
const additionalCases = [
  ["2004-03-21[u-ca=indian]", 1926, 1, "M01", 1, "shaka", 1926, "first day of leap year"],
  ["2005-03-22[u-ca=indian]", 1927, 1, "M01", 1, "shaka", 1927, "first day of common year"],
  ["2006-07-25[u-ca=islamic-umalqura]", 1427, 6, "M06", 29, "ah", 1427, "ICU4C/ICU4X discrepancy"],
  ["2025-04-19[u-ca=persian]", 1404, 1, "M01", 30, "ap", 1404, "ICU4C/ICU4X discrepancy"],
  ["2046-10-30[u-ca=hebrew]", 5807, 1, "M01", 30, "am", 5807, "ICU4C/ICU4X discrepancy"],
];

for (const [string, year, month, monthCode, day, era, eraYear, descr] of additionalCases) {
  const dateFromString = Temporal.PlainDate.from(string);
  TemporalHelpers.assertPlainDate(
    dateFromString,
    year, month, monthCode, day,
    `${descr} - created from string ${string}`,
    era, eraYear);
}
