// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.monthCode
description: monthCode should work for Chinese calendar leap dates
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const calendar = "chinese";

const leapMonthCases = [
  { year: 1971, month: 6, monthCode: "M05L", day: 1, referenceYear: 1971, referenceDay: 23 },
  { year: 1974, month: 5, monthCode: "M04L", day: 1, referenceYear: 1963, referenceDay: 22 },
  { year: 1976, month: 9, monthCode: "M08L", day: 1, referenceYear: 1957, referenceDay: 24 },
  { year: 1979, month: 7, monthCode: "M06L", day: 1, referenceYear: 1960, referenceDay: 24 },
  { year: 1982, month: 5, monthCode: "M04L", day: 1, referenceYear: 1963, referenceDay: 23 },
  { year: 1987, month: 7, monthCode: "M06L", day: 1, referenceYear: 1960, referenceDay: 24 },
  { year: 1990, month: 6, monthCode: "M05L", day: 1, referenceYear: 1971, referenceDay: 23 },
  { year: 1993, month: 4, monthCode: "M03L", day: 1, referenceYear: 1966, referenceDay: 22 },
  { year: 1995, month: 9, monthCode: "M08L", day: 1, referenceYear: 1957, referenceDay: 25 },
  { year: 1998, month: 6, monthCode: "M05L", day: 1, referenceYear: 1971, referenceDay: 24 },
  { year: 2001, month: 5, monthCode: "M04L", day: 1, referenceYear: 1963, referenceDay: 23 },
  { year: 2004, month: 3, monthCode: "M02L", day: 1, referenceYear: 1947, referenceDay: 21 },
  { year: 2006, month: 8, monthCode: "M07L", day: 1, referenceYear: 1968, referenceDay: 24 },
  { year: 2009, month: 6, monthCode: "M05L", day: 1, referenceYear: 1971, referenceDay: 23 },
  { year: 2012, month: 5, monthCode: "M04L", day: 1, referenceYear: 1963, referenceDay: 21 },
  { year: 2017, month: 7, monthCode: "M06L", day: 1, referenceYear: 1960, referenceDay: 23 },
  { year: 2020, month: 5, monthCode: "M04L", day: 1, referenceYear: 1963, referenceDay: 23 },
  { year: 2023, month: 3, monthCode: "M02L", day: 1, referenceYear: 1947, referenceDay: 22 },
  { year: 2025, month: 7, monthCode: "M06L", day: 1, referenceYear: 1960, referenceDay: 25 },
  { year: 2028, month: 6, monthCode: "M05L", day: 1, referenceYear: 1971, referenceDay: 23 },
  { year: 2031, month: 4, monthCode: "M03L", day: 1, referenceYear: 1966, referenceDay: 22 },
  { year: 2036, month: 7, monthCode: "M06L", day: 1, referenceYear: 1960, referenceDay: 23 },
  { year: 2039, month: 6, monthCode: "M05L", day: 1, referenceYear: 1971, referenceDay: 22 },
  { year: 2042, month: 3, monthCode: "M02L", day: 1, referenceYear: 1947, referenceDay: 22 },
  { year: 2044, month: 8, monthCode: "M07L", day: 1, referenceYear: 1968, referenceDay: 23 },
  { year: 2047, month: 6, monthCode: "M05L", day: 1, referenceYear: 1971, referenceDay: 23 },
];

for (var {year, month, monthCode, day } of leapMonthCases) {
  const date = Temporal.PlainDateTime.from({
    year,
    month,
    day,
    calendar
  });
  TemporalHelpers.assertPlainDateTime(date, year, month, monthCode, day, 0, 0, 0, 0, 0, 0, "constructing PlainDateTime from month number");

  const date2 = Temporal.PlainDateTime.from({
    year,
    monthCode,
    day,
    calendar
  });
  TemporalHelpers.assertPlainDateTime(date2, year, month, monthCode, day,  0, 0, 0, 0, 0, 0, "constructing PlainDateTime from month code");
  assert.sameValue(date.equals(date2), true, "datetime from month should equal datetime from month code");

  assert.throws(RangeError, () => {
    Temporal.PlainDateTime.from({
      year,
      month: 15,
      day: 1,
      calendar
    }, { overflow: "reject" });
  });

  const constrained = Temporal.PlainDateTime.from({
    year,
    month: 15,
    day: 1,
    calendar
  });
  assert.sameValue(constrained.monthCode, "M12");
}
