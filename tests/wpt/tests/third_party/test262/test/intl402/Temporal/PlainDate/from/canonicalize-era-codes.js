// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.from
description: Calendar era code is canonicalized
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const date1 = Temporal.PlainDate.from({
  calendar: "gregory",
  era: "ad",
  eraYear: 2024,
  year: 2024,
  month: 1,
  day: 1
});
TemporalHelpers.assertPlainDate(
  date1,
  2024, 1, "M01", 1,
  "'ad' is accepted as alias for 'ce'",
  "ce", 2024
);

const date2 = Temporal.PlainDate.from({
  calendar: "gregory",
  era: "bc",
  eraYear: 44,
  year: -43,
  month: 3,
  day: 15
});
TemporalHelpers.assertPlainDate(
  date2,
  -43, 3, "M03", 15,
  "'bc' is accepted as alias for 'bce'",
  "bce", 44
);
