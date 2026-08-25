// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.locale
description: Verifies the behavior of an undefined numeric option to the Locale constructor.
info: |
    Intl.Locale( tag [, options] )

    ...
    24. Let kn be ? GetOption(options, "numeric", "boolean", undefined, undefined).
    25. If kn is not undefined, set kn to ! ToString(kn).
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

const options = { numeric: undefined };
assert.sameValue(
  new Intl.Locale('en', options).toString(),
  "en",
  'new Intl.Locale("en", {numeric: undefined}).toString() returns "en"'
);

assert.sameValue(
  new Intl.Locale('en-u-kn-true', options).toString(),
  "en-u-kn",
  'new Intl.Locale("en-u-kn-true", {numeric: undefined}).toString() returns "en-u-kn"'
);

assert.sameValue(
  new Intl.Locale('en-u-kf-lower', options).numeric,
  false,
  'The value of new Intl.Locale("en-u-kf-lower", {numeric: undefined}).numeric equals `false`'
);
