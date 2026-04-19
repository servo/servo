// Copyright (C) 2020 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.getcanonicallocales
description: >
  No RangeError is thrown when a language tag includes a valid transformed extension subtag.
info: |
  8.2.1 Intl.getCanonicalLocales (locales)
    1. Let ll be ? CanonicalizeLocaleList(locales).
    2. Return CreateArrayFromList(ll).

  9.2.1 CanonicalizeLocaleList (locales)
    ...
    7. Repeat, while k < len
      ...
      c. If kPresent is true, then
        ...
        v. If IsStructurallyValidLanguageTag(tag) is false, throw a RangeError exception.
        vi. Let canonicalizedTag be CanonicalizeUnicodeLocaleId(tag).
        ...

includes: [testIntl.js]
---*/

const valid = [
  // tlang with unicode_language_subtag.
  "en-t-en",

  // tlang with unicode_script_subtag.
  "en-t-en-latn",

  // tlang with unicode_region_subtag.
  "en-t-en-ca",

  // tlang with unicode_script_subtag and unicode_region_subtag.
  "en-t-en-latn-ca",

  // tlang with unicode_variant_subtag.
  "en-t-en-emodeng",

  // tlang with unicode_script_subtag and unicode_variant_subtag.
  "en-t-en-latn-emodeng",

  // tlang with unicode_script_subtag and unicode_variant_subtag.
  "en-t-en-ca-emodeng",

  // tlang with unicode_script_subtag, unicode_region_subtag, and unicode_variant_subtag.
  "en-t-en-latn-ca-emodeng",

  // No tlang. (Must contain at least one tfield.)
  "en-t-d0-ascii",
];

const extraFields = [
  // No extra tfield
  "",

  // tfield with a tvalue consisting of a single subtag.
  "-i0-handwrit",

  // tfield with a tvalue consisting of two subtags.
  "-s0-accents-publish",
];

for (let tag of valid) {
  for (let extra of extraFields) {
    let actualTag = tag + extra;

    // Make sure the test data is correct.
    assert(isCanonicalizedStructurallyValidLanguageTag(actualTag),
           "\"" + actualTag + "\" is a canonical and structurally valid language tag.");

    let result = Intl.getCanonicalLocales(actualTag);
    assert.sameValue(result.length, 1);
    assert.sameValue(result[0], actualTag);
  }
}
