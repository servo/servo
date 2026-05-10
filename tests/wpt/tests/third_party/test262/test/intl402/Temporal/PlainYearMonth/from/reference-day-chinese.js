// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.calendar.prototype.yearmonthfromfields
description: Reference ISO day is chosen to be the first of the calendar month
info: |
  6.d. Perform ! CreateDataPropertyOrThrow(_fields_, *"day"*, *1*<sub>𝔽</sub>).
    e. Let _result_ be ? CalendarDateToISO(_calendar_.[[Identifier]], _fields_, _options_).
includes: [temporalHelpers.js]
features: [Temporal]
---*/

// Month codes, month indices, and the ISO reference days of the months in 2022
const months2022TestData = [
  ["M01", 1, 1],
  ["M02", 2, 3],
  ["M03", 3, 1],
  ["M04", 4, 1],
  ["M05", 5, 30],
  ["M06", 6, 29],
  ["M07", 7, 29],
  ["M08", 8, 27],
  ["M09", 9, 26],
  ["M10", 10, 25],
  ["M11", 11, 24],
  ["M12", 12, 23],
];
for (let [nonLeapMonthCode, month, referenceISODay] of months2022TestData) {
  const leapMonthCode = nonLeapMonthCode + "L";
  const fields = { year: 2022, monthCode: leapMonthCode, calendar: "chinese" };

  const result = Temporal.PlainYearMonth.from(fields, { overflow: "constrain" });
  TemporalHelpers.assertPlainYearMonth(
    result,
    2022, month, nonLeapMonthCode,
    `Chinese intercalary month ${leapMonthCode} is constrained to ${nonLeapMonthCode} in year 2022 (overflow constrain)`,
    /* era = */ undefined, /* era year = */ undefined, referenceISODay
  );

  assert.throws(
    RangeError,
    () => Temporal.PlainYearMonth.from(fields, { overflow: "reject" }),
    `Chinese intercalary month ${leapMonthCode} does not exist in year 2022 (overflow reject)`
  );
}

// Years in which leap months exist in our supported range according to
// ICU4C/ICU4X. Note use of near-future year for "M11L" rather than use of a
// year several hundred years before the start of our range.
const leapMonthsTestData = [
  ["M02L", 2023, 3, 22],
  ["M03L", 1993, 4, 22],
  ["M04L", 2020, 5, 23],
  ["M05L", 2009, 6, 23],
  ["M06L", 2017, 7, 23],
  ["M07L", 2006, 8, 24],
  ["M08L", 1995, 9, 25],
  ["M09L", 2014, 10, 24],
  ["M10L", 1984, 11, 23],
  ["M11L", 2033, 12, 22],
];
for (const [monthCode, year, month, referenceISODay, isoYear = year, isoMonth = month] of leapMonthsTestData) {
  const result = Temporal.PlainYearMonth.from({ year, monthCode, calendar: "chinese" });
  TemporalHelpers.assertPlainYearMonth(
    result,
    year, month, monthCode,
    `Date of sample Chinese intercalary month ${monthCode}`,
    /* era = */ undefined, /* era year = */ undefined, referenceISODay
  );
  const isoYearMonth = result.toString().slice(0, 7);
  assert.sameValue(isoYearMonth, `${isoYear}-${String(isoMonth).padStart(2, '0')}`, `${year}-${monthCode} starts in ISO month ${isoYear}-${isoMonth}`);
}
