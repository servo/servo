// Copyright 2025 Richard Gibson. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.locale
description: >
    Checks error cases for the options argument to the Locale
    constructor.
info: |
    Intl.Locale( tag [, options] )
    12. Set _tag_ to ? UpdateLanguageId(_tag_, _options_).

    UpdateLanguageId ( tag, options )
    8. Let _variants_ be ? GetOption(_options_, *"variants"*, ~string~, ~empty~, GetLocaleVariants(_baseName_)).
    9. If _variants_ is not *undefined*, then
      a. If _variants_ cannot be matched by the <code>unicode_variant_subtag</code> Unicode locale nonterminal, throw a *RangeError* exception.

features: [Intl.Locale]
---*/

/*
 unicode_variant_subtag = (alphanum{5,8} | digit alphanum{3})
*/
const invalidVariantsOptions = [
  "",
  "a",
  "1",
  "ab",
  "2x",
  "abc",
  "3xy",
  "abcd",
  "abcdefghi",

  // Value contains more than just the 'variants' production.
  "GB-scouse",

  // Value contains duplicates.
  "fonipa-fonipa",
  "fonipa-valencia-Fonipa",

  // Value contains out-of-place dashes.
  "-",
  "-spanglis",
  "spanglis-",
  "-spanglis-oxendict",
  "spanglis-oxendict-",
  "spanglis--oxendict",
];
for (const variants of invalidVariantsOptions) {
  assert.throws(RangeError, function() {
    new Intl.Locale("en", {variants});
  }, `new Intl.Locale("en", {variants: "${variants}"}) throws RangeError`);
}
