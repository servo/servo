// Copyright (C) 2020 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.getcanonicallocales
description: >
  "kb", "kc", "kh", "kk", and "kn" Unicode extension keys canonicalise "yes" to "true".
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

  UTS 35, §3.2.1 Canonical Unicode Locale Identifiers
    Use the bcp47 data to replace keys, types, tfields, and tvalues by their canonical forms.
    See Section 3.6.4 U Extension Data Files) and Section 3.7.1 T Extension Data Files. The
    aliases are in the alias attribute value, while the canonical is in the name attribute value.

  UTS 35, §3.2.1 Canonical Unicode Locale Identifiers
    Any type or tfield value "true" is removed.
includes: [testIntl.js]
---*/

const unicodeKeys = [
  // <key name="kb" [...] alias="colBackwards">
  //   <type name="true" [...] alias="yes"/>
  "kb",

  // <key name="kc" [...] alias="colCaseLevel">
  //   <type name="true" [...] alias="yes"/>
  "kc",

  // <key name="kh" [...] alias="colBackwards">
  //   <type name="true" [...] alias="yes"/>
  "kh",

  // <key name="kh" [...] alias="colHiraganaQuaternary">
  //   <type name="true" [...] alias="yes"/>
  "kk",

  // <key name="kn" [...] alias="colNumeric">
  //   <type name="true" [...] alias="yes"/>
  "kn",
];

for (let key of unicodeKeys) {
  let tag = `und-u-${key}-yes`;
  let canonical = `und-u-${key}`;

  // Make sure the test data is correct.
  assert.sameValue(isCanonicalizedStructurallyValidLanguageTag(tag), false,
                   "\"" + tag + "\" isn't a canonical language tag.");
  assert(isCanonicalizedStructurallyValidLanguageTag(canonical),
         "\"" + canonical + "\" is a canonical and structurally valid language tag.");

  let result = Intl.getCanonicalLocales(tag);
  assert.sameValue(result.length, 1);
  assert.sameValue(result[0], canonical);
}

// Test some other Unicode extension keys which don't contain an alias entry to
// canonicalise "yes" to "true".
const otherUnicodeKeys = [
  "ka", "kf", "kr", "ks", "kv",
];

for (let key of otherUnicodeKeys) {
  let tag = `und-u-${key}-yes`;

  // Make sure the test data is correct.
  assert(isCanonicalizedStructurallyValidLanguageTag(tag),
         "\"" + tag + "\" is a canonical and structurally valid language tag.");

  let result = Intl.getCanonicalLocales(tag);
  assert.sameValue(result.length, 1);
  assert.sameValue(result[0], tag);
}
