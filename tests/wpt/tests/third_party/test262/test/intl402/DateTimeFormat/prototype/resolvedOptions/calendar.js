// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.datetimeformat.prototype.resolvedoptions
description: Verifies that the calendar option is respected.
locale: [en]
features: [Intl.Era-monthcode]
---*/

const tests = [
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

for (const calendar of tests) {
  const formatter = new Intl.DateTimeFormat("en", { calendar });
  const options = formatter.resolvedOptions();
  assert.sameValue(options.calendar, calendar, "Resolved calendar");
}

const aliases = [
  ["ethiopic-amete-alem", "ethioaa"],
  ["islamicc", "islamic-civil"],
];

for (const [alias, calendar] of aliases) {
  const formatter = new Intl.DateTimeFormat("en", { calendar: alias });
  const options = formatter.resolvedOptions();
  assert.sameValue(options.calendar, calendar, "Resolved alias " + alias);
}
