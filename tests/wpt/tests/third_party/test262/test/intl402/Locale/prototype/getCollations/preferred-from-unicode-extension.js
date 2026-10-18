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

const languages = [
  "und",
  "en",
];

const extensions = [
  // uvalue with a valid collation.
  "big5han",
  "compat",
  "dict",
  "direct",
  "ducet",
  "emoji",
  "eor",
  "gb2312",
  "phonebk",
  "phonetic",
  "pinyin",
  "reformed",
  "search",
  "searchjl",
  "standard",
  "stroke",
  "trad",
  "unihan",
  "zhuyin",

  // uvalue with an invalid collation.
  "abcdefgh",
  "dictdict",
  "reform",
];

for (let language of languages) {
  for (let extension of extensions) {
    let locale = `${language}-u-co-${extension}`;
    let loc = new Intl.Locale(locale)
    let collations = loc.getCollations();

    assert.sameValue(
      loc.collation,
      extension,
      `collation with locale "${locale}"`
    );

    assert.compareArray(
      collations,
      [extension],
      `collation with locale "${locale}"`
    );
  }
}
