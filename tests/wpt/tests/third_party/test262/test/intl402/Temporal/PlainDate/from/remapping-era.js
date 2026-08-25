// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.from
description: Test remapping behaviour of regnal eras
info: |
  CalendarDateFromFields:
  1. Perform ? CalendarResolveFields(_calendar_, _fields_, _date_).
  2. Let result be ? CalendarDateToISO(_calendar_, _fields_, _overflow_).

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
    return Temporal.PlainDate.from({ ...fields, calendar: "japanese" }, { overflow });
  }

  // Reiwa era started in month 5 of the year, so the era of month 1 is remapped
  // to be the correct one for the month.
  TemporalHelpers.assertPlainDate(
    test({ era: "reiwa", eraYear: 1, monthCode: "M01", day: 24 }),
    2019, 1, "M01", 24,
    "Reiwa 1 before May is mapped to Heisei 31",
    "heisei", 31
  );

  // Reiwa era started on the first day of the month, so day 1 does not need
  // remapping.
  TemporalHelpers.assertPlainDate(
    test({ era: "reiwa", eraYear: 1, monthCode: "M05", day: 1 }),
    2019, 5, "M05", 1,
    "05-01 Reiwa 1 is not remapped",
    "reiwa", 1
  );

  // Heisei era started on the eighth day of the month. So on previous days the
  // era is remapped to be the correct one for the day.
  TemporalHelpers.assertPlainDate(
    test({ era: "heisei", eraYear: 1, monthCode: "M01", day: 4 }),
    1989, 1, "M01", 4,
    "01-04 Heisei 1 is remapped to 01-04 Showa 64",
    "showa", 64
  );

  // Era year past the end of the Heisei era is remapped to Reiwa era
  TemporalHelpers.assertPlainDate(
    test({ era: "heisei", eraYear: 37, monthCode: "M04", day: 25 }),
    2025, 4, "M04", 25,
    "Heisei 37 is remapped to Reiwa 7",
    "reiwa", 7
  );

  // Zero year in a 1-based era is remapped to the previous era
  TemporalHelpers.assertPlainDate(
    test({ era: "reiwa", eraYear: 0, monthCode: "M04", day: 25 }),
    2018, 4, "M04", 25,
    "Reiwa 0 is remapped to Heisei 30",
    "heisei", 30
  );

  // Negative year in a forward-counting era is remapped to the previous era
  TemporalHelpers.assertPlainDate(
    test({ era: "reiwa", eraYear: -20, monthCode: "M04", day: 25 }),
    1998, 4, "M04", 25,
    "Reiwa -20 is remapped to Heisei 10",
    "heisei", 10
  );

  // Test the last two things for Gregorian eras as well
  function testGregorian(fields) {
    return Temporal.PlainDate.from({ ...fields, calendar: "gregory" }, { overflow });
  }
  TemporalHelpers.assertPlainDate(
    testGregorian({ era: "ce", eraYear: 0, monthCode: "M04", day: 25 }),
    0, 4, "M04", 25,
    "0 CE is remapped to 1 BCE",
    "bce", 1
  );
  TemporalHelpers.assertPlainDate(
    testGregorian({ era: "ce", eraYear: -20, monthCode: "M04", day: 25 }),
    -20, 4, "M04", 25,
    "-20 CE is remapped to 21 BCE",
    "bce", 21
  );

  // Zero year in a backwards-counting era is remapped to the next era
  TemporalHelpers.assertPlainDate(
    testGregorian({ era: "bce", eraYear: 0, monthCode: "M04", day: 25 }),
    1, 4, "M04", 25,
    "0 BCE is remapped to 1 CE",
    "ce", 1
  );

  // Negative year in a backwards-counting era is remapped to the next era
  TemporalHelpers.assertPlainDate(
    testGregorian({ era: "bce", eraYear: -20, monthCode: "M04", day: 25 }),
    21, 4, "M04", 25,
    "-20 BCE is remapped to 21 CE",
    "ce", 21
  );
}
