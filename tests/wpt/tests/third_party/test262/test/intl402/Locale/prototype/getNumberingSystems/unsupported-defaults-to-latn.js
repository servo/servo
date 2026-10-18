// Copyright 2026 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.locale.prototype.getNumberingSystems
description: >
  Defaults to "latn" if the locale is not supported by Intl.NumberFormat.
info: |
  NumberingSystemsOfLocale ( loc )
  ...
  4. Else,
    a. Let list be « "latn" ».
  5. Return CreateArrayFromList(list).
features: [Intl.Locale, Intl.Locale-info]
---*/

const locales = [
  // Undetermined language.
  "und",

  // Klingon as a sample constructed language.
  "tlh",

  // "qfz" as a CLDR private use, excluded language subtag.
  "qfz",
];

for (let locale of locales) {
  // Skip if the locale is (somehow) supported by this implementation.
  if (Intl.NumberFormat.supportedLocalesOf(locale).length > 0) {
    continue;
  }

  let numberingSystems = new Intl.Locale(locale).getNumberingSystems();

  assert.compareArray(
    numberingSystems,
    ["latn"],
    `with locale "${locale}"`
  );
}
