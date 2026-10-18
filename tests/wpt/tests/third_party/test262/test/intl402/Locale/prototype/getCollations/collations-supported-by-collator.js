// Copyright 2026 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.locale.prototype.getCollations
description: >
  The returned collations are supported by Intl.Collator.
info: |
  CollationsOfLocale ( loc )
  ...
  3. If language is not "und", then
    a. Let r be LookupMatchingLocaleByPrefix(%Intl.Collator%.[[AvailableLocales]], « loc.[[Locale]] »).
    b. If r is not undefined, then
      i. Let foundLocale be r.[[locale]].
    ...
    d. Let foundLocaleData be %Intl.Collator%.[[SortLocaleData]].[[<foundLocale>]].
    e. Let list be a copy of foundLocaleData.[[co]].
    ...
  ...
  6. Return CreateArrayFromList(sorted).
features: [Intl.Locale, Intl.Locale-info]
---*/

const locales = [
  "en",

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
  // Skip if this implementation doesn't have data for |locale|.
  if (Intl.Collator.supportedLocalesOf(locale).length === 0) {
    continue;
  }

  let collations = new Intl.Locale(locale).getCollations();
  for (let collation of collations) {
    assert.sameValue(
      new Intl.Collator(locale, {collation}).resolvedOptions().collation,
      collation,
      `with locale "${locale}"`
    );
  }
}
