// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.monthCode
description: monthCode should work for Dangi calendar leap dates
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const calendar = "dangi";

const leapMonthCases = [
  { year: 1971, month: 6, monthCode: "M05L", day: 1, referenceYear: 1971, referenceDay: 23 },
  { year: 1974, month: 5, monthCode: "M04L", day: 1, referenceYear: 1963, referenceDay: 22 },
  { year: 1976, month: 9, monthCode: "M08L", day: 1, referenceYear: 1957, referenceDay: 24 },
  { year: 1979, month: 7, monthCode: "M06L", day: 1, referenceYear: 1960, referenceDay: 24 },
  { year: 1982, month: 5, monthCode: "M04L", day: 1, referenceYear: 1963, referenceDay: 23 },
  // See https://github.com/tc39/proposal-intl-era-monthcode/issues/60
  // { year: 1984, month: 11, monthCode: "M10L", day: 1, referenceYear: 1870, referenceDay: 23 },
  { year: 1987, month: 7, monthCode: "M06L", day: 1, referenceYear: 1960, referenceDay: 26 },
  { year: 1990, month: 6, monthCode: "M05L", day: 1, referenceYear: 1971, referenceDay: 23 },
  { year: 1993, month: 4, monthCode: "M03L", day: 1, referenceYear: 1966, referenceDay: 22 },
  { year: 1995, month: 9, monthCode: "M08L", day: 1, referenceYear: 1957, referenceDay: 25 },
  { year: 1998, month: 6, monthCode: "M05L", day: 1, referenceYear: 1971, referenceDay: 24 },
  { year: 2001, month: 5, monthCode: "M04L", day: 1, referenceYear: 1963, referenceDay: 23 },
  { year: 2004, month: 3, monthCode: "M02L", day: 1, referenceYear: 1947, referenceDay: 21 },
  { year: 2006, month: 8, monthCode: "M07L", day: 1, referenceYear: 1968, referenceDay: 24 },
  { year: 2009, month: 6, monthCode: "M05L", day: 1, referenceYear: 1971, referenceDay: 23 },
  { year: 2012, month: 4, monthCode: "M03L", day: 1, referenceYear: 1966, referenceDay: 21 },
  // See https://github.com/tc39/proposal-intl-era-monthcode/issues/60
  // { year: 2014, month: 10, monthCode: "M09L", day: 1, referenceYear: 1832, referenceDay: 24 },
  { year: 2017, month: 6, monthCode: "M05L", day: 1, referenceYear: 1971, referenceDay: 24 },
  { year: 2020, month: 5, monthCode: "M04L", day: 1, referenceYear: 1963, referenceDay: 23 },
  { year: 2023, month: 3, monthCode: "M02L", day: 1, referenceYear: 1947, referenceDay: 22 },
  { year: 2025, month: 7, monthCode: "M06L", day: 1, referenceYear: 1960, referenceDay: 25 },
  { year: 2028, month: 6, monthCode: "M05L", day: 1, referenceYear: 1971, referenceDay: 23 },
  { year: 2031, month: 4, monthCode: "M03L", day: 1, referenceYear: 1966, referenceDay: 22 },
  // See https://github.com/tc39/proposal-intl-era-monthcode/issues/60
  // { year: 2033, month: 12, monthCode: "M11L", day: 1, referenceYear: 1813, referenceDay: 22 },
  { year: 2036, month: 7, monthCode: "M06L", day: 1, referenceYear: 1960, referenceDay: 23 },
  { year: 2039, month: 6, monthCode: "M05L", day: 1, referenceYear: 1971, referenceDay: 22 },
  { year: 2042, month: 3, monthCode: "M02L", day: 1, referenceYear: 1947, referenceDay: 22 },
  { year: 2044, month: 8, monthCode: "M07L", day: 1, referenceYear: 1968, referenceDay: 23 },
  { year: 2047, month: 6, monthCode: "M05L", day: 1, referenceYear: 1971, referenceDay: 23 },
];

for (var {year, month, monthCode, referenceDay, } of leapMonthCases) {
  const ym = Temporal.PlainYearMonth.from({
    year,
    month,
    calendar
  });
  TemporalHelpers.assertPlainYearMonth(ym, year, month, monthCode, "constructing PlainYearMonth from month number", undefined, undefined, referenceDay);

  const ym2 = Temporal.PlainYearMonth.from({
    year,
    monthCode,
    calendar
  });
  TemporalHelpers.assertPlainYearMonth(ym2, year, month, monthCode, "constructing PlainYearMonth from month code", undefined, undefined, referenceDay);
  assert.sameValue(ym.equals(ym2), true, "year-month from month should equal year-month from month code");

  assert.throws(RangeError, () => {
    Temporal.PlainYearMonth.from({
      year,
      month: 15,
      calendar
    }, { overflow: "reject" });
  });

  const constrained = Temporal.PlainYearMonth.from({
    year,
    month: 15,
    calendar
  });
  assert.sameValue(constrained.monthCode, "M12");
}
