// Copyright 2026 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.Locale.prototype.getNumberingSystems
description: >
  Matches the resolved numbering system from Intl.NumberFormat using the "lookup" matching.
info: |
  NumberingSystemsOfLocale ( loc )
  ...
  2. Let match be LookupMatchingLocaleByPrefix(%Intl.NumberFormat%.[[AvailableLocales]], « loc.[[Locale]] »).
  3. If match is not undefined, then
    ...
  4. Else,
    a. Let list be « "latn" ».
  5. Return CreateArrayFromList(list).

features: [Intl.Locale, Intl.Locale-info]
---*/

const locales = [
  "und-BD",
  "und-SA",
];

for (let locale of locales) {
  // Skip if the locale is (somehow) supported by this implementation.
  if (Intl.NumberFormat.supportedLocalesOf(locale, {localeMatcher: "lookup"}).length > 0) {
    continue;
  }

  let numberingSystems = new Intl.Locale(locale).getNumberingSystems();

  assert.compareArray(
    numberingSystems,
    ["latn"],
    `with locale "${locale}"`
  );
}
