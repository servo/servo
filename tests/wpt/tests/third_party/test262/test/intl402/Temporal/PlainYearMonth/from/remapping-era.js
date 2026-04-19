// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.from
description: Test remapping behaviour of regnal eras
info: |
  CalendarYearMonthFromFields:
  1. Perform ? CalendarResolveFields(_calendar_, _fields_, ~year-month~).
  2. Let _firstDayIndex_ be the 1-based index of the first day of the month
     described by _fields_ (i.e., 1 unless the month's first day is skipped by
     this calendar.)
  3. Set _fields_.[[Day]] to _firstDayIndex_.
  4. Let result be ? CalendarDateToISO(_calendar_, _fields_, _overflow_).

  CalendarResolveFields:
  When the fields of _fields_ are inconsistent with respect to a non-unset
  _fields_.[[Era]], it is recommended that _fields_.[[Era]] and
  _fields_.[[EraYear]] be updated to resolve the inconsistency by lenient
  interpretation of out-of-bounds values (rather than throwing a *RangeError*),
  which is particularly useful for consistent interpretation of dates in
  calendars with regnal eras.

includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

// Based on a test originally by André Bargull <andre.bargull@gmail.com>

// Notes:
// - "heisei" era started January 8, 1989.
// - "reiwa" era started May 1, 2019.

for (const overflow of ["constrain", "reject"]) {
  function test(fields) {
    return Temporal.PlainYearMonth.from({ ...fields, calendar: "japanese" }, { overflow });
  }

  // Reiwa era started in month 5 of the year, so the era of month 1 is remapped
  // to be the correct one for the month.
  TemporalHelpers.assertPlainYearMonth(
    test({ era: "reiwa", eraYear: 1, monthCode: "M01" }),
    2019, 1, "M01",
    "Reiwa 1 before May is mapped to Heisei 31",
    "heisei", 31, /* reference day = */ 1
  );

  // Reiwa era started on the first day of the month, so the reference day 1
  // does not need remapping.
  TemporalHelpers.assertPlainYearMonth(
    test({ era: "reiwa", eraYear: 1, monthCode: "M05" }),
    2019, 5, "M05",
    "reference day is 1",
    "reiwa", 1, /* reference day = */ 1
  );

  // Heisei era started on the eighth day of the month, but PlainYearMonth
  // references the first day of the month. So the era is remapped to be the
  // correct one for the reference day.
  TemporalHelpers.assertPlainYearMonth(
    test({ era: "heisei", eraYear: 1, monthCode: "M01" }),
    1989, 1, "M01",
    "01-01 Heisei 1 is remapped to 01-01 Showa 64",
    "showa", 64, /* reference day = */ 1
  );

  // Era year past the end of the Heisei era is remapped to Reiwa era
  TemporalHelpers.assertPlainYearMonth(
    test({ era: "heisei", eraYear: 37, monthCode: "M04" }),
    2025, 4, "M04",
    "Heisei 37 is remapped to Reiwa 7",
    "reiwa", 7, /* reference day = */ 1
  );

  // Zero year in a 1-based era is remapped to the previous era
  TemporalHelpers.assertPlainYearMonth(
    test({ era: "reiwa", eraYear: 0, monthCode: "M04" }),
    2018, 4, "M04",
    "Reiwa 0 is remapped to Heisei 30",
    "heisei", 30, /* reference day = */ 1
  );

  // Negative year in a forwards-counting era is remapped to the previous era
  TemporalHelpers.assertPlainYearMonth(
    test({ era: "reiwa", eraYear: -20, monthCode: "M04" }),
    1998, 4, "M04",
    "Reiwa -20 is remapped to Heisei 10",
    "heisei", 10, /* reference day = */ 1
  );

  // Test the last two things for Gregorian eras as well
  function testGregorian(fields) {
    return Temporal.PlainYearMonth.from({ ...fields, calendar: "gregory" }, { overflow });
  }
  TemporalHelpers.assertPlainYearMonth(
    testGregorian({ era: "ce", eraYear: 0, monthCode: "M04" }),
    0, 4, "M04",
    "0 CE is remapped to 1 BCE",
    "bce", 1, /* reference day = */ 1
  );
  TemporalHelpers.assertPlainYearMonth(
    testGregorian({ era: "ce", eraYear: -20, monthCode: "M04" }),
    -20, 4, "M04",
    "-20 CE is remapped to 21 BCE",
    "bce", 21, /* reference day = */ 1
  );

  // Zero year in a backwards-counting era is remapped to the next era
  TemporalHelpers.assertPlainYearMonth(
    testGregorian({ era: "bce", eraYear: 0, monthCode: "M04" }),
    1, 4, "M04",
    "0 BCE is remapped to 1 CE",
    "ce", 1, /* reference day = */ 1
  );

  // Negative year in a backwards-counting era is remapped to the next era
  TemporalHelpers.assertPlainYearMonth(
    testGregorian({ era: "bce", eraYear: -20, monthCode: "M04" }),
    21, 4, "M04",
    "-20 BCE is remapped to 21 CE",
    "ce", 21, /* reference day = */ 1
  );
}
