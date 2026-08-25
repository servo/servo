// Copyright 2018 André Bargull; Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.locale
description: >
    Verifies canonicalization, minimization and maximization of specific tags.
info: |
    ApplyOptionsToTag( tag, options )
    10. Return CanonicalizeLanguageTag(tag).

    Intl.Locale.prototype.maximize ()
    3. Let maximal be the result of the Add Likely Subtags algorithm applied to loc.[[Locale]].

    Intl.Locale.prototype.minimize ()
    3. Let minimal be the result of the Remove Likely Subtags algorithm applied to loc.[[Locale]].
features: [Intl.Locale]
---*/

// Test some language tags where we know that either CLDR or ICU produce
// different results compared to the canonicalization specified in RFC 5646.
var testData = [
    {
        tag: "mo",
        canonical: "ro",
        maximized: "ro-Latn-RO",
    },
    {
        tag: "es-ES-preeuro",
        maximized: "es-Latn-ES-preeuro",
        minimized: "es-preeuro",
    },
    {
        tag: "uz-UZ-cyrillic",
        maximized: "uz-Latn-UZ-cyrillic",
        minimized: "uz-cyrillic",
    },
    {
        tag: "posix",
    },
    {
        tag: "hi-direct",
        maximized: "hi-Deva-IN-direct",
    },
    {
        tag: "zh-pinyin",
        maximized: "zh-Hans-CN-pinyin",
    },
    {
        tag: "zh-stroke",
        maximized: "zh-Hans-CN-stroke",
    },
    {
        tag: "aar-x-private",
        // "aar" should be canonicalized into "aa" because "aar" matches the type attribute of
        // a languageAlias element in 
        // https://www.unicode.org/repos/cldr/trunk/common/supplemental/supplementalMetadata.xml
        canonical: "aa-x-private",
        maximized: "aa-Latn-ET-x-private",
   },
    {
        tag: "heb-x-private",
        // "heb" should be canonicalized into "he" because "heb" matches the type attribute of
        // a languageAlias element in 
        // https://www.unicode.org/repos/cldr/trunk/common/supplemental/supplementalMetadata.xml
        canonical: "he-x-private",
        maximized: "he-Hebr-IL-x-private",
    },
    {
        tag: "de-u-kf",
        maximized: "de-Latn-DE-u-kf",
    },
    {
        tag: "ces",
        // "ces" should be canonicalized into "cs" because "ces" matches the type attribute of
        // a languageAlias element in 
        // https://www.unicode.org/repos/cldr/trunk/common/supplemental/supplementalMetadata.xml
        canonical: "cs",
        maximized: "cs-Latn-CZ",
    },
    {
        tag: "hy-arevela",
        canonical: "hy",
        maximized: "hy-Armn-AM",
    },
    {
        tag: "hy-arevmda",
        canonical: "hyw",
        maximized: "hyw-Armn-AM",
    },
];

for (const {tag, canonical = tag, maximized = canonical, minimized = canonical} of testData) {
    const loc = new Intl.Locale(tag);
    assert.sameValue(
      new Intl.Locale(tag).toString(),
      canonical,
      `new Intl.Locale("${tag}").toString() returns "${canonical}"`
    );
    assert.sameValue(
      new Intl.Locale(tag).maximize().toString(),
      maximized,
      `new Intl.Locale("${tag}").maximize().toString() returns "${maximized}"`
    );
    assert.sameValue(
      new Intl.Locale(tag).minimize().toString(),
      minimized,
      `new Intl.Locale("${tag}").minimize().toString() returns "${minimized}"`
    );
}
