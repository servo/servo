// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.getcanonicallocales
description: >
  Call Intl.getCanonicalLocales function with valid language tags.
info: |
  8.2.1 Intl.getCanonicalLocales (locales)
    1. Let ll be ? CanonicalizeLocaleList(locales).
    2. Return CreateArrayFromList(ll).

  9.2.1 CanonicalizeLocaleList (locales)
    ...
    7. Repeat, while k < len
      a. Let Pk be ToString(k).
      b. Let kPresent be ? HasProperty(O, Pk).
      c. If kPresent is true, then
        i. Let kValue be ? Get(O, Pk).
        ...
        iii. Let tag be ? ToString(kValue).
        ...
        v. Let canonicalizedTag be CanonicalizeLanguageTag(tag).
        vi. If canonicalizedTag is not an element of seen, append canonicalizedTag as the last element of seen.
      ...
includes: [testIntl.js]
---*/

var canonicalizedTags = {
  "de": "de",
  "DE-de": "de-DE",
  "de-DE": "de-DE",
  "cmn": "zh",
  "CMN-hANS": "zh-Hans",
  "cmn-hans-cn": "zh-Hans-CN",
  "es-419": "es-419",
  "es-419-u-nu-latn": "es-419-u-nu-latn",
  "cmn-hans-cn-u-ca-t-ca-x-t-u": "zh-Hans-CN-t-ca-u-ca-x-t-u",
  "de-gregory-u-ca-gregory": "de-gregory-u-ca-gregory",
  "sgn-GR": "gss",
  "ji": "yi",
  "de-DD": "de-DE",
  "in": "id",
  "sr-cyrl-ekavsk": "sr-Cyrl-ekavsk",
  "en-ca-newfound": "en-CA-newfound",
  "sl-rozaj-biske-1994": "sl-1994-biske-rozaj",
  "da-u-attr": "da-u-attr",
  "da-u-attr-co-search": "da-u-attr-co-search",
};

// make sure the data above is correct
Object.getOwnPropertyNames(canonicalizedTags).forEach(function (tag) {
  var canonicalizedTag = canonicalizedTags[tag];
  assert(
    isCanonicalizedStructurallyValidLanguageTag(canonicalizedTag),
    "Test data \"" + canonicalizedTag + "\" is not canonicalized and structurally valid language tag."
  );
});

Object.getOwnPropertyNames(canonicalizedTags).forEach(function (tag) {
  var canonicalLocales = Intl.getCanonicalLocales(tag);
  assert.sameValue(canonicalLocales.length, 1);
  assert.sameValue(canonicalLocales[0], canonicalizedTags[tag]);
});
