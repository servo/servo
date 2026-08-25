// Copyright (C) 2021 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.supportedvaluesof
description: >
  The returned "collation" values can be used with Collator.
info: |
  Intl.supportedValuesOf ( key )

  1. Let key be ? ToString(key).
  ...
  3. Else if key is "collation", then
    a. Let list be ! AvailableCollations( ).
  ...
  9. Return ! CreateArrayFromList( list ).

  AvailableCollations ( )
    The AvailableCollations abstract operation returns a List, ordered as if an
    Array of the same values had been sorted using %Array.prototype.sort% using
    undefined as comparefn, that contains unique collation types identifying the
    collations for which the implementation provides the functionality of
    Intl.Collator objects.
includes: [testIntl.js]
locale: [en, ar, de, es, ko, ln, si, sv, zh]
features: [Intl-enumeration, Array.prototype.includes]
---*/

const collations = Intl.supportedValuesOf("collation");

// Not all locales support all possible collations, so test the minimal set to
// cover all supported collations.
//
// The list of all collations can be derived from
// <https://github.com/unicode-org/cldr/blob/master/common/bcp47/collation.xml>.
//
// Note: "standard" and "search" are explicitly disallowed by Intl.Collator.
const locales = [
  "en", // ducet, emoji, eor
  "ar", // compat
  "de", // phonebk
  "es", // trad
  "hi", // direct
  "ko", // searchjl
  "ln", // phonetic
  "si", // dict
  "sv", // reformed
  "zh", // big5han, gb2312, pinyin, stroke, unihan, zhuyin
];

for (let collation of collations) {
  let supported = false;
  for (let locale of locales) {
    let obj = new Intl.Collator(locale, {collation});
    if (obj.resolvedOptions().collation === collation) {
      supported = true;
      break;
    }
  }

  assert(supported, `${collation} is supported by Collator`);
}

for (let collation of allCollations()) {
  let supported = false;
  for (let locale of locales) {
    let obj = new Intl.Collator(locale, {collation});
    if (obj.resolvedOptions().collation === collation) {
      supported = true;
      break;
    }
  }

  if (supported) {
    assert(collations.includes(collation),
           `${collation} supported but not returned by supportedValuesOf`);
  } else {
    assert(!collations.includes(collation),
           `${collation} not supported but returned by supportedValuesOf`);
  }
}
