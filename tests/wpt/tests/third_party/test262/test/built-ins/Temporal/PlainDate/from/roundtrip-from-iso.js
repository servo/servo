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

const year2000 = new Temporal.PlainDate(2000, 1, 1);
testRoundtrip(year2000);
const year1 = new Temporal.PlainDate(1, 1, 1);
testRoundtrip(year1);

function testRoundtrip(date) {
  const dateFromYearMonth = Temporal.PlainDate.from({
    year: date.year,
    month: date.month,
    day: date.day,
  });
  TemporalHelpers.assertPlainDate(
    dateFromYearMonth,
    date.year, date.month, date.monthCode, date.day,
    `${date} - created from year and month`);

  const dateFromYearMonthCode = Temporal.PlainDate.from({
    year: date.year,
    monthCode: date.monthCode,
    day: date.day,
  });
  TemporalHelpers.assertPlainDate(
    dateFromYearMonthCode,
    date.year, date.month, date.monthCode, date.day,
    `${date} - created from year and month code`);
}
