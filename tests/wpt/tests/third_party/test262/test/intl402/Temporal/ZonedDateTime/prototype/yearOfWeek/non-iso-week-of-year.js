// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.yearofweek
description: >
  Temporal.ZonedDateTime.prototype.yearOfWeek returns undefined for all
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
    new Temporal.ZonedDateTime(1_704_112_496_987_654_321n, "UTC", calendar).yearOfWeek,
    undefined,
    `${calendar} does not provide week numbers`
  );
}
