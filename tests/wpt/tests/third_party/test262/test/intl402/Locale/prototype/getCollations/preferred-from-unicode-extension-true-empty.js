// Copyright 2026 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.locale.prototype.getCollations
description: >
  Returns the preferred collations from the "co" Unicode extension.
info: |
  CollationsOfLocale ( loc )
  1. If loc.[[Collation]] is not undefined, then
    a. Return CreateArrayFromList(« loc.[[Collation]] »).
  ...
features: [Intl.Locale, Intl.Locale-info]
---*/

const locales = [
  "und-u-co",
  "und-u-co-true",
  "en-u-co",
  "en-u-co-true",
]

for (let locale of locales) {
  let loc = new Intl.Locale(locale);
  let collations = loc.getCollations();

  assert.sameValue(
    loc.collation,
    "",
    `collation with locale "${locale}"`
  );

  assert.compareArray(
    collations,
    [""],
    `collations with locale "${locale}"`
  );
}
