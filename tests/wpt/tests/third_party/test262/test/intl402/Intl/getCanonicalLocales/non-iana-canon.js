// Copyright 2018 André Bargull; Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

// Slip from intl402/Locale/constructor-non-iana-canon.js
/*---
esid: sec-intl.getcanonicallocales
description: >
    Verifies canonicalization, of specific tags.
info: |
    ApplyOptionsToTag( tag, options )
    10. Return CanonicalizeLanguageTag(tag).
---*/

// Test some language tags where we know that either CLDR or ICU produce
// different results compared to the canonicalization specified in RFC 5646.
var testData = [
    {
        tag: "mo",
        canonical: "ro",
    },
    {
        tag: "es-ES-preeuro",
    },
    {
        tag: "uz-UZ-cyrillic",
    },
    {
        tag: "posix",
    },
    {
        tag: "hi-direct",
    },
    {
        tag: "zh-pinyin",
    },
    {
        tag: "zh-stroke",
    },
    {
        tag: "aar-x-private",
        // "aar" should be canonicalized into "aa" because "aar" matches the type attribute of
        // a languageAlias element in 
        // https://www.unicode.org/repos/cldr/trunk/common/supplemental/supplementalMetadata.xml
        canonical: "aa-x-private", 
    },
    {
        tag: "heb-x-private",
        // "heb" should be canonicalized into "he" because "heb" matches the type attribute of
        // a languageAlias element in 
        // https://www.unicode.org/repos/cldr/trunk/common/supplemental/supplementalMetadata.xml
        canonical: "he-x-private",
    },
    {
        tag: "de-u-kf",
    },
    {
        tag: "ces",
        // "ces" should be canonicalized into "cs" because "ces" matches the type attribute of
        // a languageAlias element in 
        // https://www.unicode.org/repos/cldr/trunk/common/supplemental/supplementalMetadata.xml
        canonical: "cs",
    },
    {
        tag: "hy-arevela",
        canonical: "hy",
    },
    {
        tag: "hy-arevmda",
        canonical: "hyw",
    },
];

for (const {tag, canonical = tag} of testData) {
    assert.sameValue(
      Intl.getCanonicalLocales(tag)[0],
      canonical,
      'The value of Intl.getCanonicalLocales(tag)[0] equals the value of `canonical`'
    );
}
