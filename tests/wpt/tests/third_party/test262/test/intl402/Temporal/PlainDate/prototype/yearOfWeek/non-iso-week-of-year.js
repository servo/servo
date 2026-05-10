// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.yearofweek
description: >
  Temporal.PlainDate.prototype.yearOfWeek returns undefined for all
  non-ISO calendars without a well-defined week numbering system.
features: [Temporal, Intl.Era-monthcode]
---*/

const nonIsoCalendars = [
  "buddhist",
  "chinese",
  "coptic",
  "dangi",
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
  "roc"
];


for (const calendar of nonIsoCalendars) {
  assert.sameValue(
    new Temporal.PlainDate(2024, 1, 1, calendar).yearOfWeek,
    undefined,
    `${calendar} does not provide week numbers`
  );
}

