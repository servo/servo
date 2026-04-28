// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.calendar.prototype.calendaryearmonthfromfields
description: >
  Reference ISO day is chosen to be the first of the calendar month
  See https://github.com/tc39/proposal-temporal/issues/3150 for more context.
info: |
  1. Let _firstDayIndex_ be the 1-based index of the first day of the month described by _fields_ (i.e., 1 unless the month's first day is skipped by this calendar.)
  2. Set _fields_.[[Day]] to _firstDayIndex_.
  3. Perform ? CalendarResolveFields(_calendar_, _fields_, ~year-month~).
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const result1 = Temporal.PlainYearMonth.from({calendar: "japanese", era: "heisei", eraYear: 1, month: 1});
TemporalHelpers.assertPlainYearMonth(
  result1,
  1989, 1, "M01",
  "era is corrected based on reference day (Heisei begins on January 8)",
  "showa", 64
);

const result2 = Temporal.PlainYearMonth.from({calendar: "japanese", era: "showa", eraYear: 1, month: 12});
TemporalHelpers.assertPlainYearMonth(
  result2,
  1926, 12, "M12",
  "era is corrected based on reference day (Showa begins on December 25)",
  "taisho", 15
);

const result3 = Temporal.PlainYearMonth.from({calendar: "japanese", era: "taisho", eraYear: 1, month: 7});
TemporalHelpers.assertPlainYearMonth(
  result3,
  1912, 7, "M07",
  "era is corrected based on reference day (Taishō begins on July 30)",
  "meiji", 45
);
