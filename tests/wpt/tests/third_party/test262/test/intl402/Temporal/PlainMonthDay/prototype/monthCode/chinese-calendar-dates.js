// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.prototype.monthCode
description: monthCode should work for Chinese calendar leap dates
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const calendar = "chinese";

const leapMonthCases = [
  { year: 1971, month: 6, monthCode: "M05L", day: 1, referenceYear: 1971 },
  { year: 1974, month: 5, monthCode: "M04L", day: 1, referenceYear: 1963 },
  { year: 1976, month: 9, monthCode: "M08L", day: 1, referenceYear: 1957 },
  { year: 1979, month: 7, monthCode: "M06L", day: 1, referenceYear: 1960 },
  { year: 1982, month: 5, monthCode: "M04L", day: 1, referenceYear: 1963 },
  // See https://github.com/tc39/proposal-intl-era-monthcode/issues/60
  // { year: 1984, month: 11, monthCode: "M10L", day: 1, referenceYear: 1870 },
  { year: 1987, month: 7, monthCode: "M06L", day: 1, referenceYear: 1960 },
  { year: 1990, month: 6, monthCode: "M05L", day: 1, referenceYear: 1971 },
  { year: 1993, month: 4, monthCode: "M03L", day: 1, referenceYear: 1966 },
  { year: 1995, month: 9, monthCode: "M08L", day: 1, referenceYear: 1957 },
  { year: 1998, month: 6, monthCode: "M05L", day: 1, referenceYear: 1971 },
  { year: 2001, month: 5, monthCode: "M04L", day: 1, referenceYear: 1963 },
  { year: 2004, month: 3, monthCode: "M02L", day: 1, referenceYear: 1947 },
  { year: 2006, month: 8, monthCode: "M07L", day: 1, referenceYear: 1968 },
  { year: 2009, month: 6, monthCode: "M05L", day: 1, referenceYear: 1971 },
  { year: 2012, month: 5, monthCode: "M04L", day: 1, referenceYear: 1963 },
  // See https://github.com/tc39/proposal-intl-era-monthcode/issues/60
  // { year: 2014, month: 10, monthCode: "M09L", day: 1, referenceYear: 1832 },
  { year: 2017, month: 7, monthCode: "M06L", day: 1, referenceYear: 1960 },
  { year: 2020, month: 5, monthCode: "M04L", day: 1, referenceYear: 1963 },
  { year: 2023, month: 3, monthCode: "M02L", day: 1, referenceYear: 1947 },
  { year: 2025, month: 7, monthCode: "M06L", day: 1, referenceYear: 1960 },
  { year: 2028, month: 6, monthCode: "M05L", day: 1, referenceYear: 1971 },
  { year: 2031, month: 4, monthCode: "M03L", day: 1, referenceYear: 1966 },
  // See https://github.com/tc39/proposal-intl-era-monthcode/issues/60
  // { year: 2033, month: 12, monthCode: "M11L", day: 1, referenceYear: 1813 },
  { year: 2036, month: 7, monthCode: "M06L", day: 1, referenceYear: 1960 },
  { year: 2039, month: 6, monthCode: "M05L", day: 1, referenceYear: 1971 },
  { year: 2042, month: 3, monthCode: "M02L", day: 1, referenceYear: 1947 },
  { year: 2044, month: 8, monthCode: "M07L", day: 1, referenceYear: 1968 },
  { year: 2047, month: 6, monthCode: "M05L", day: 1, referenceYear: 1971 }
];

assert.throws(RangeError, () => {
  Temporal.PlainMonthDay.from({
    monthCode: "M15",
    day: 1,
    calendar
  }, { overflow: "reject" });
});

assert.throws(RangeError, () => {
  Temporal.PlainMonthDay.from({
    monthCode: "M15",
    day: 1,
    calendar
  });
});

for (var {year, month, monthCode, day, referenceYear} of leapMonthCases) {
  const md = Temporal.PlainMonthDay.from({
    year,
    month,
    day,
    calendar
  });
  TemporalHelpers.assertPlainMonthDay(md, monthCode, day, "md", referenceYear);

  const md2 = Temporal.PlainMonthDay.from({
    monthCode,
    day,
    calendar
  });
  TemporalHelpers.assertPlainMonthDay(md2, monthCode, day, "md2", referenceYear);
  assert.sameValue(md.equals(md2), true);

  assert.throws(RangeError, () => {
    Temporal.PlainMonthDay.from({
      year,
      month: 15,
      day: 1,
      calendar
    }, { overflow: "reject" });
  });

  const constrained = Temporal.PlainMonthDay.from({
    year,
    month: 15,
    day: 1,
    calendar
  });
  assert.sameValue(constrained.monthCode, "M12");
}
