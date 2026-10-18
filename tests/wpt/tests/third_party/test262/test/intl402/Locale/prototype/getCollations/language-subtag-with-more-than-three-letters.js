// Copyright 2026 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.Locale.prototype.getCollations
description: >
  Language subtags with more than three letters are supported.
info: |
  CollationsOfLocale ( loc )
  1. If loc.[[Collation]] is not undefined, then
    a. Return CreateArrayFromList(« loc.[[Collation]] »).
  2. Let _match_ be LookupMatchingLocaleByPrefix(%Intl.Collator%.[[AvailableLocales]], « _loc_.[[Locale]] »).
  3. If _match_ is not *undefined*, then
    ...
  4. Else,
    1. Let _list_ be « *"emoji"*, *"eor"* ».
  5. Let _sorted_ be a copy of _list_, sorted according to lexicographic code unit order.
  6. Return CreateArrayFromList(sorted).
features: [Intl.Locale, Intl.Locale-info]
---*/

// "abcdefgh" is not a registered language, so it always returns the default
// locale's collations.
assert.compareArray(
  new Intl.Locale("abcdefgh").getCollations(),
  ["emoji", "eor"],
  `with locale "abcdefgh"`
);

// Except if an explicit "co" Unicode extension subtag is present.
assert.compareArray(
  new Intl.Locale("abcdefgh-u-co-phonebk").getCollations(),
  ["phonebk"],
  `with locale "abcdefgh-u-co-phonebk"`
);
