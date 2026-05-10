// Copyright 2018 André Bargull; Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.locale
description: >
    Verifies canonicalization of specific tags.
info: |
    ApplyOptionsToTag( tag, options )
    2. If IsStructurallyValidLanguageTag(tag) is false, throw a RangeError exception.
    ...
    13. Return CanonicalizeLanguageTag(tag).
features: [Intl.Locale]
---*/

const validLanguageTags = {
  "eN": "en",
  "en-gb": "en-GB",
  "IT-LATN-iT": "it-Latn-IT",
  "th-th-u-nu-thai": "th-TH-u-nu-thai",
  "en-x-u-foo": "en-x-u-foo",
  "en-a-bar-x-u-foo": "en-a-bar-x-u-foo",
  "en-x-u-foo-a-bar": "en-x-u-foo-a-bar",
  "en-u-baz-a-bar-x-u-foo": "en-a-bar-u-baz-x-u-foo",
  "DE-1996": "de-1996", // unicode_language_subtag sep unicode_variant_subtag
  
  // unicode_language_subtag (sep unicode_variant_subtag)*
  "sl-ROZAJ-BISKE-1994": "sl-1994-biske-rozaj",
  "zh-latn-pinyin-pinyin2": "zh-Latn-pinyin-pinyin2",
};

for (const [langtag, canonical] of Object.entries(validLanguageTags)) {
  assert.sameValue(
    new Intl.Locale(canonical).toString(),
    canonical,
    `new Intl.Locale("${canonical}").toString() returns "${canonical}"`
  );
  assert.sameValue(
    new Intl.Locale(langtag).toString(),
    canonical,
    `new Intl.Locale("${langtag}").toString() returns "${canonical}"`
  );
}

// unicode_language_subtag = alpha{2,3} | alpha{5,8};
const invalidLanguageTags = [
  "X-u-foo", 
  "Flob",
  "ZORK",
  "Blah-latn",
  "QuuX-latn-us",
  "SPAM-gb-x-Sausages-BACON-eggs",
];

for (const langtag of invalidLanguageTags) {
  assert.throws(RangeError, () => new Intl.Locale(langtag));
}
