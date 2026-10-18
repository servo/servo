// Copyright 2026 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.Locale.prototype.getNumberingSystems
description: >
  Language subtags with more than three letters are supported.
info: |
  NumberingSystemsOfLocale ( loc )
  1. If loc.[[NumberingSystem]] is not undefined, then
    a. Return CreateArrayFromList(« loc.[[NumberingSystem]] »).
  2. Let match be LookupMatchingLocaleByPrefix(%Intl.NumberFormat%.[[AvailableLocales]], « loc.[[Locale]] »).
  3. If match is not undefined, then
    ...
  4. Else,
    a. Let list be « "latn" ».
  5. Return CreateArrayFromList(list).

features: [Intl.Locale, Intl.Locale-info]
---*/

// "abcdefgh" is not a registered language, so it always returns the default
// numbering system "latn".
assert.compareArray(
  new Intl.Locale("abcdefgh").getNumberingSystems(),
  ["latn"],
  `with locale "abcdefgh"`
);

// Except if an explicit "nu" Unicode extension subtag is present.
assert.compareArray(
  new Intl.Locale("abcdefgh-u-nu-arab").getNumberingSystems(),
  ["arab"],
  `with locale "abcdefgh-u-nu-arab"`
);
