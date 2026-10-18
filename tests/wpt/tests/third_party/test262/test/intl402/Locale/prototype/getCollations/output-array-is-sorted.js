// Copyright 2026 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.locale.prototype.getCollations
description: >
  The returned array is sorted in lexicographic code unit order.
info: |
  CollationsOfLocale ( loc )
  ...
  5. Let sorted be a copy of list, sorted according to lexicographic code unit order.
  6. Return CreateArrayFromList(sorted).
features: [Intl.Locale, Intl.Locale-info]
---*/

const locales = [
  "en",
  "und",

  // Locales which have additional collations defined in CLDR.
  "ar",
  "bn",
  "de",
  "es",
  "fi",
  "ja",
  "kn",
  "ko",
  "ln",
  "si",
  "sv",
  "vi",
  "zh",
];

for (let locale of locales) {
  let collations = new Intl.Locale(locale).getCollations();
  let sorted = collations.slice(0).sort();

  assert.compareArray(collations, sorted, `with locale "${locale}"`);
}
