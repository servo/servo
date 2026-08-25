// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.from
description: Calendar era code is canonicalized
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const date1 = Temporal.PlainYearMonth.from({
  calendar: "gregory",
  era: "ad",
  eraYear: 2024,
  year: 2024,
  month: 1,
});
TemporalHelpers.assertPlainYearMonth(
  date1,
  2024, 1, "M01",
  "'ad' is accepted as alias for 'ce'",
  "ce", 2024
);

const date2 = Temporal.PlainYearMonth.from({
  calendar: "gregory",
  era: "bc",
  eraYear: 44,
  year: -43,
  month: 3,
});
TemporalHelpers.assertPlainYearMonth(
  date2,
  -43, 3, "M03",
  "'bc' is accepted as alias for 'bce'",
  "bce", 44
);
