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
    ...
    13. If _variants_ is not *undefined*, set _newTag_ to the string-concatenation of _newTag_, *"-"*, and _variants_.

features: [Intl.Locale]
---*/

const validVariantsOptions = [
  ['en', undefined, undefined],
  ['en', 'spanglis', 'en-spanglis'],

  // unicode_variant_subtag = (alphanum{5,8} | digit alphanum{3})
  ['xx', '1xyz', 'xx-1xyz'],
  ['xx', '1234', 'xx-1234'],
  ['xx', 'abcde', 'xx-abcde'],
  ['xx', '12345678', 'xx-12345678'],
  ['xx', '1xyz-1234-abcde-12345678', 'xx-1234-12345678-1xyz-abcde'],

  // Canonicalization affects subtag ordering.
  ['en', 'spanglis-oxendict', 'en-oxendict-spanglis'],
];
for (const [lang, variants, baseName] of validVariantsOptions) {
  let options = { variants };
  let optionsRepr = `{variants: ${typeof variants === "string" ? `"${variants}"` : variants}}`;
  let instance;
  let expect;

  instance = new Intl.Locale(lang, options);
  expect = baseName || lang;
  assert.sameValue(instance.toString(), expect,
    `new Intl.Locale("${lang}", ${optionsRepr}).toString() returns "${expect}"`);

  instance = new Intl.Locale(lang + '-fonipa', options);
  expect = baseName || (lang + '-fonipa');
  assert.sameValue(instance.toString(), expect,
    `new Intl.Locale("${lang}-fonipa", ${optionsRepr}).toString() returns "${expect}"`);

  instance = new Intl.Locale(lang + '-u-ca-gregory', options);
  expect = (baseName || lang) + '-u-ca-gregory';
  assert.sameValue(instance.toString(), expect,
    `new Intl.Locale("${lang}-u-ca-gregory", ${optionsRepr}).toString() returns "${expect}"`);

  instance = new Intl.Locale(lang + '-fonipa-u-ca-gregory', options);
  expect = (baseName || (lang + '-fonipa')) + '-u-ca-gregory';
  assert.sameValue(instance.toString(), expect,
    `new Intl.Locale("${lang}-fonipa-u-ca-gregory", ${optionsRepr}).toString() returns "${expect}"`);
}
