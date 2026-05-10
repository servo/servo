// Copyright (C) 2020 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.getcanonicallocales
description: >
  A RangeError is thrown when a language tag includes an invalid transformed extension subtag.
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
        ...

includes: [testIntl.js]
---*/

const invalid = [
  // empty
  "en-t",
  "en-t-a",
  "en-t-x",
  "en-t-0",

  // incomplete
  "en-t-",
  "en-t-en-",
  "en-t-0x-",

  // tlang: unicode_language_subtag must be 2-3 or 5-8 characters and mustn't
  // contain extlang subtags.
  "en-t-root",
  "en-t-abcdefghi",
  "en-t-ar-aao",

  // tlang: unicode_script_subtag must be 4 alphabetical characters, can't
  // be repeated.
  "en-t-en-lat0",
  "en-t-en-latn-latn",

  // tlang: unicode_region_subtag must either be 2 alpha characters or a three
  // digit code.
  "en-t-en-0",
  "en-t-en-00",
  "en-t-en-0x",
  "en-t-en-x0",
  "en-t-en-latn-0",
  "en-t-en-latn-00",
  "en-t-en-latn-xyz",

  // tlang: unicode_variant_subtag is either 5-8 alphanum characters or 4
  // characters starting with a digit.
  "en-t-en-abcdefghi",
  "en-t-en-latn-gb-ab",
  "en-t-en-latn-gb-abc",
  "en-t-en-latn-gb-abcd",
  "en-t-en-latn-gb-abcdefghi",

  // tkey must be followed by tvalue.
  "en-t-d0",
  "en-t-d0-m0",
  "en-t-d0-x-private",
];

for (let tag of invalid) {
  // Make sure the test data is correct.
  assert.sameValue(isCanonicalizedStructurallyValidLanguageTag(tag), false,
                   "\"" + tag + "\" isn't a structurally valid language tag.");

  assert.throws(RangeError, () => Intl.getCanonicalLocales(tag), `${tag}`);
}
