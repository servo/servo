// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.from
description: >
  Check that various dates created from calculated properties have the expected
  properties
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const options = { overflow: "reject" };

const calendars = [
  "buddhist",
  "chinese",
  "coptic",
  "dangi",
  "ethioaa",
  "ethiopic",
  "gregory",
  "hebrew",
  "indian",
  "islamic-civil",
  "islamic-tbla",
  "islamic-umalqura",
  "japanese",
  "persian",
  "roc",
];

for (const calendar of calendars) {
  const year2000 = new Temporal.PlainDate(2000, 1, 1, calendar);
  testRoundtrip(year2000, calendar, "ISO date 2000-01-01");
  const year1 = new Temporal.PlainDate(1, 1, 1, calendar);
  testRoundtrip(year1, calendar, "ISO date 0001-01-01");
}

// Additional cases that were moved in from staging tests, or that we add to
// catch regressions
const additionalCases = [
  ["indian", 2000, 12, 31, "https://github.com/unicode-org/icu4x/issues/4914"],
];

for (const [calendar, isoYear, isoMonth, isoDay, descr] of additionalCases) {
  const date = new Temporal.PlainDate(isoYear, isoMonth, isoDay, calendar);
  testRoundtrip(date, calendar, descr);
}

function testRoundtrip(date, calendar, descr) {
  const dateFromYearMonth = Temporal.PlainDate.from({
    calendar,
    year: date.year,
    month: date.month,
    day: date.day,
  });
  TemporalHelpers.assertPlainDate(
    dateFromYearMonth,
    date.year, date.month, date.monthCode, date.day,
    `${descr} (${calendar}) - created from year and month`,
    date.era, date.eraYear);

  const dateFromYearMonthCode = Temporal.PlainDate.from({
    calendar,
    year: date.year,
    monthCode: date.monthCode,
    day: date.day,
  });
  TemporalHelpers.assertPlainDate(
    dateFromYearMonthCode,
    date.year, date.month, date.monthCode, date.day,
    `${descr} (${calendar}) - created from year and month code`,
    date.era, date.eraYear);

  if (date.era === undefined) return;  // skip era-less calendars

  const dateFromEraMonth = Temporal.PlainDate.from({
    calendar,
    era: date.era,
    eraYear: date.eraYear,
    month: date.month,
    day: date.day,
  });
  TemporalHelpers.assertPlainDate(
    dateFromEraMonth,
    date.year, date.month, date.monthCode, date.day,
    `${descr} (${calendar}) - created from era, era year, and month`,
    date.era, date.eraYear);

  const dateFromEraMonthCode = Temporal.PlainDate.from({
    calendar,
    era: date.era,
    eraYear: date.eraYear,
    monthCode: date.monthCode,
    day: date.day,
  });
  TemporalHelpers.assertPlainDate(
    dateFromEraMonthCode,
    date.year, date.month, date.monthCode, date.day,
    `${descr} (${calendar}) - created from era, era year, and month code`,
    date.era, date.eraYear);
}
