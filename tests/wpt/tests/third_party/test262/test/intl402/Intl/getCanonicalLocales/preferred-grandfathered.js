// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.getcanonicallocales
description: >
  Call Intl.getCanonicalLocales function with grandfathered language tags.
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
        v. Let canonicalizedTag be CanonicalizeLanguageTag(tag).
      ...

  6.2.3 CanonicalizeLanguageTag ( locale )
    The CanonicalizeLanguageTag abstract operation returns the canonical and case-regularized form
    of the locale argument (which must be a String value that is a structurally valid Unicode
    BCP 47 Locale Identifier as verified by the IsStructurallyValidLanguageTag abstract operation).
    A conforming implementation shall take the steps specified in the “BCP 47 Language Tag to
    Unicode BCP 47 Locale Identifier” algorithm, from Unicode Technical Standard #35 LDML
    § 3.3.1 BCP 47 Language Tag Conversion.

includes: [testIntl.js]
---*/

// Generated from http://www.iana.org/assignments/language-subtag-registry/language-subtag-registry
// File-Date: 2017-08-15

var irregularGrandfathered = [
  "en-gb-oed",
  "i-ami",
  "i-bnn",
  "i-default",
  "i-enochian",
  "i-hak",
  "i-klingon",
  "i-lux",
  "i-mingo",
  "i-navajo",
  "i-pwn",
  "i-tao",
  "i-tay",
  "i-tsu",
  "sgn-be-fr",
  "sgn-be-nl",
  "sgn-ch-de",
];

var regularGrandfatheredNonUTS35 = [
  "no-bok",
  "no-nyn",
  "zh-min",
  "zh-min-nan",
];

var regularGrandfatheredUTS35 = {
  "art-lojban": "jbo",
  "cel-gaulish": "xtg",
  "zh-guoyu": "zh",
  "zh-hakka": "hak",
  "zh-xiang": "hsn",
};

// make sure the data above is correct
irregularGrandfathered.forEach(function (tag) {
  assert.sameValue(
    isCanonicalizedStructurallyValidLanguageTag(tag), false,
    "Test data \"" + tag + "\" is not a structurally valid language tag."
  );
});
regularGrandfatheredNonUTS35.forEach(function (tag) {
  assert.sameValue(
    isCanonicalizedStructurallyValidLanguageTag(tag), false,
    "Test data \"" + tag + "\" is not a structurally valid language tag."
  );
});
Object.getOwnPropertyNames(regularGrandfatheredUTS35).forEach(function (tag) {
  var canonicalizedTag = regularGrandfatheredUTS35[tag];
  assert(
    isCanonicalizedStructurallyValidLanguageTag(canonicalizedTag),
    "Test data \"" + canonicalizedTag + "\" is a canonicalized and structurally valid language tag."
  );
});

Object.getOwnPropertyNames(regularGrandfatheredUTS35).forEach(function (tag) {
  var canonicalLocales = Intl.getCanonicalLocales(tag);
  assert.sameValue(canonicalLocales.length, 1);
  assert.sameValue(canonicalLocales[0], regularGrandfatheredUTS35[tag]);
});
