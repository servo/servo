// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.from
description: RangeError thrown if era is invalid for this calendar
features: [Temporal]
---*/

const calendarsWithEras = [
  "buddhist",
  "coptic",
  "ethioaa",
  "ethiopic",
  "gregory",
  "hebrew",
  "indian",
  "islamic-civil",
  "islamic-tbla",
  "islamic-umalqura",
  "japanese",
  "persian",
  "roc",
];

const calendarsWithoutEras = [
  "chinese",
  "dangi",
];

calendarsWithEras.forEach((calendar) => {
  // "xyz" is not a valid era in any supported calendar
  assert.throws(RangeError,
    () => Temporal.PlainDateTime.from({ year: 2025, month: 1, day: 1, hour: 12, minute: 34, era: "xyz", eraYear: 2025, calendar }),
    `xyz is not a valid era in calendar ${calendar}`);
});

calendarsWithoutEras.forEach((calendar) => {
  // era is ignored
  const result = Temporal.PlainDateTime.from({ year: 2025, month: 1, day: 1, hour: 12, minute: 34, era: "xyz", eraYear: 2025, calendar });
  assert.sameValue(result instanceof Temporal.PlainDateTime, true, `era should be ignored for calendar ${calendar}`);
});
