// Copyright 2018 André Bargull; Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.locale
description: >
    Checks error cases for the options argument to the Locale
    constructor.
info: |
    Intl.Locale( tag [, options] )
    10. If options is undefined, then
    11. Else
        a. Let options be ? ToObject(options).
    12. Set tag to ? ApplyOptionsToTag(tag, options).

    ApplyOptionsToTag( tag, options )
    ...
    4. If language is not undefined, then
        a. If language does not match the language production, throw a RangeError exception.
    ...

features: [Intl.Locale]
---*/

/*
 language      = 2*3ALPHA            ; shortest ISO 639 code
                 ["-" extlang]       ; sometimes followed by
                                     ; extended language subtags
               / 4ALPHA              ; or reserved for future use
               / 5*8ALPHA            ; or registered language subtag

 extlang       = 3ALPHA              ; selected ISO 639 codes
                 *2("-" 3ALPHA)      ; permanently reserved
*/
const invalidLanguageOptions = [
  "",
  "a",
  "ab7",
  "notalanguage",
  "undefined",

  // "root" is treated as a special `unicode_language_subtag`, but is not
  // actually one and is not valid in a Unicode BCP 47 locale identifier.
  // https://unicode.org/reports/tr35/#unicode_bcp47_locale_id
  "root",

  // Value contains more than just the 'language' production.
  "fr-Latn",
  "fr-FR",
  "sa-vaidika",
  "fr-a-asdf",
  "fr-x-private",

  // Irregular grandfathered language tag.
  "i-klingon",

  // Regular grandfathered language tag.
  "zh-min",
  "zh-min-nan",

  // Reserved with extended language subtag
  "abcd-US",
  "abcde-US",
  "abcdef-US",
  "abcdefg-US",
  "abcdefgh-US",

  7,
];
for (const language of invalidLanguageOptions) {
  assert.throws(RangeError, function() {
    new Intl.Locale("en", {language});
  }, `new Intl.Locale("en", {language: "${language}"}) throws RangeError`);
}
