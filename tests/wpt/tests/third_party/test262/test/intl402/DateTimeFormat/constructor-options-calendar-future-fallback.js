// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.datetimeformat
description: >
  Tests that fallbacks for not-yet-adopted calendars are selected from one of
  the values returned from `AvailableCalendars`.
locale: [en]
includes: [temporalHelpers.js]
features: [Intl.Era-monthcode]
---*/

const availableCalendars = [
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
  "iso8601",
  "japanese",
  "persian",
  "roc",
];

for (const calendar of TemporalHelpers.NotYetSupportedCalendars) {
  const option = new Intl.DateTimeFormat("en", { calendar });
  assert(
    availableCalendars.includes(option.resolvedOptions().calendar),
    `${calendar} should fall back to an available calendar`
  );

  const uExtension = new Intl.DateTimeFormat(`en-u-ca-${calendar}`);
  assert(
    availableCalendars.includes(uExtension.resolvedOptions().calendar),
    `${calendar} should fall back to an available calendar`
  );
}
