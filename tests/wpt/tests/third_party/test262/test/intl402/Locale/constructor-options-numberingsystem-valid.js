// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.locale
description: >
    Checks valid cases for the options argument to the Locale constructor.
info: |
    Intl.Locale( tag [, options] )

    ...
    27. Let numberingSystem be ? GetOption(options, "numberingSystem", "string", undefined, undefined).
    ...
    29. Set opt.[[nu]] to numberingSystem.
    ...
    30. Let r be ! ApplyUnicodeExtensionToTag(tag, opt, relevantExtensionKeys).
    ...

    ApplyUnicodeExtensionToTag( tag, options, relevantExtensionKeys )

    ...
    8. Let locale be the String value that is tag with all Unicode locale extension sequences removed.
    9. Let newExtension be ! CanonicalizeUnicodeExtension(attributes, keywords).
    10. If newExtension is not the empty String, then
        a. Let locale be ! InsertUnicodeExtension(locale, newExtension).
    ...

    CanonicalizeUnicodeExtension( attributes, keywords )
    ...
    4. Repeat for each element entry of keywords in List order,
        a. Let keyword be entry.[[Key]].
        b. If entry.[[Value]] is not the empty String, then
            i. Let keyword be the string-concatenation of keyword, "-", and entry.[[Value]].
        c. Append keyword to fullKeywords.
    ...
features: [Intl.Locale]
---*/

const validNumberingSystemOptions = [
  ["abc", "en-u-nu-abc"],
  ["abcd", "en-u-nu-abcd"],
  ["abcde", "en-u-nu-abcde"],
  ["abcdef", "en-u-nu-abcdef"],
  ["abcdefg", "en-u-nu-abcdefg"],
  ["abcdefgh", "en-u-nu-abcdefgh"],
  ["12345678", "en-u-nu-12345678"],
  ["1234abcd", "en-u-nu-1234abcd"],
  ["1234abcd-abc123", "en-u-nu-1234abcd-abc123"],
];
for (const [numberingSystem, expected] of validNumberingSystemOptions) {
  assert.sameValue(
    new Intl.Locale('en', { numberingSystem }).toString(),
    expected,
    `new Intl.Locale("en", { numberingSystem: ${numberingSystem} }).toString() returns "${expected}"`
  );
  assert.sameValue(
    new Intl.Locale('en-u-nu-latn', { numberingSystem }).toString(),
    expected,
    `new Intl.Locale("en-u-nu-latn", { numberingSystem: ${numberingSystem} }).toString() returns "${expected}"`
  );
}
