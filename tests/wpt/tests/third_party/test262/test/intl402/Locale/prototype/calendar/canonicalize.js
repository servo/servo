// Copyright 2024 Google Inc.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-apply-unicode-extension-to-tag
description: Checks canonicalize value of extension in ApplyUnicodeExtensionToTag.
info: |
    ApplyUnicodeExtensionToTag
     1. Let _optionsUValue_ be the ASCII-lowercase of _optionsValue_.
     1. Set _value_ to the String value resulting from canonicalizing _optionsUValue_ as a value of key _key_ per <a href="https://unicode.org/reports/tr35/#processing-localeids">Unicode Technical Standard #35 Part 1 Core, Annex C LocaleId Canonicalization Section 5 Canonicalizing Syntax, Processing LocaleIds</a>.
features: [Intl.Locale]
---*/

const loc = new Intl.Locale('en', {calendar: 'islamicc'});

assert.sameValue(loc.toString(), "en-u-ca-islamic-civil",
    "'islamicc' should be canonicalize to 'islamic-civil'");

assert.sameValue(loc.calendar, "islamic-civil",
    "'islamicc' should be canonicalize to 'islamic-civil'");
