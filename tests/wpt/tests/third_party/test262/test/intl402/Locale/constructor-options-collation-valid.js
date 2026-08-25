// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.locale
description: >
    Checks valid cases for the options argument to the Locale constructor.
info: |
    Intl.Locale( tag [, options] )

    ...
    17. Let collation be ? GetOption(options, "collation", "string", undefined, undefined).
    ...
    19. Set opt.[[co]] to collation.
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

const validCollationOptions = [
  ["abc", "en-u-co-abc"],
  ["abcd", "en-u-co-abcd"],
  ["abcde", "en-u-co-abcde"],
  ["abcdef", "en-u-co-abcdef"],
  ["abcdefg", "en-u-co-abcdefg"],
  ["abcdefgh", "en-u-co-abcdefgh"],
  ["12345678", "en-u-co-12345678"],
  ["1234abcd", "en-u-co-1234abcd"],
  ["1234abcd-abc123", "en-u-co-1234abcd-abc123"],
];
for (const [collation, expected] of validCollationOptions) {
  assert.sameValue(
    new Intl.Locale('en', {collation}).toString(),
    expected,
    `new Intl.Locale('en', {collation: "${collation}"}).toString() returns "${expected}"`
  );

  assert.sameValue(
    new Intl.Locale('en-u-co-gregory', {collation}).toString(),
    expected,
    `new Intl.Locale('en-u-co-gregory', {collation: "${collation}"}).toString() returns "${expected}"`
  );
}
