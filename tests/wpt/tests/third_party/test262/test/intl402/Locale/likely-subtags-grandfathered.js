// Copyright 2018 André Bargull; Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.locale
description: >
    Verifies canonicalization, minimization and maximization of specific tags.
info: |
    ApplyOptionsToTag( tag, options )

    2. If IsStructurallyValidLanguageTag(tag) is false, throw a RangeError exception.

    9. Set tag to CanonicalizeLanguageTag(tag).

    CanonicalizeLanguageTag( tag )

    The CanonicalizeLanguageTag abstract operation returns the canonical and
    case-regularized form of the locale argument (which must be a String value
    that is a structurally valid Unicode BCP 47 Locale Identifier as verified by
    the IsStructurallyValidLanguageTag abstract operation).

    IsStructurallyValidLanguageTag ( locale )

    The IsStructurallyValidLanguageTag abstract operation verifies that the
    locale argument (which must be a String value)

    represents a well-formed Unicode BCP 47 Locale Identifier" as specified in
    Unicode Technical Standard 35 section 3.2, or successor,


    Intl.Locale.prototype.maximize ()
    3. Let maximal be the result of the Add Likely Subtags algorithm applied to loc.[[Locale]].

    Intl.Locale.prototype.minimize ()
    3. Let minimal be the result of the Remove Likely Subtags algorithm applied to loc.[[Locale]].
features: [Intl.Locale]
---*/

const irregularGrandfathered = [
    "en-GB-oed",
    "i-ami",
    "i-bnn",
    "i-default",
    "i-enochian",
    "i-hak",
    "i-klingon",
    "i-lux",
    "i-mingo",
    "i-navajo",
    "i-pwn",
    "i-tao",
    "i-tay",
    "i-tsu",
    "sgn-BE-FR",
    "sgn-BE-NL",
    "sgn-CH-DE",
];

for (const tag of irregularGrandfathered) {
    assert.throws(RangeError, () => new Intl.Locale(tag));
}

const regularGrandfathered = [
    {
        tag: "art-lojban",
        canonical: "jbo",
        maximized: "jbo-Latn-001",
    },
    {
        tag: "cel-gaulish",
        canonical: "xtg",
    },
    {
        tag: "zh-guoyu",
        canonical: "zh",
        maximized: "zh-Hans-CN",
    },
    {
        tag: "zh-hakka",
        canonical: "hak",
        maximized: "hak-Hans-CN",
    },
    {
        tag: "zh-xiang",
        canonical: "hsn",
        maximized: "hsn-Hans-CN",
    },
];

for (const {tag, canonical, maximized = canonical, minimized = canonical} of regularGrandfathered) {
    const loc = new Intl.Locale(tag);
    assert.sameValue(loc.toString(), canonical);

    assert.sameValue(loc.maximize().toString(), maximized);
    assert.sameValue(loc.maximize().maximize().toString(), maximized);

    assert.sameValue(loc.minimize().toString(), minimized);
    assert.sameValue(loc.minimize().minimize().toString(), minimized);

    assert.sameValue(loc.maximize().minimize().toString(), minimized);
    assert.sameValue(loc.minimize().maximize().toString(), maximized);
}

const regularGrandfatheredWithExtLang = [
    "no-bok",
    "no-nyn",
    "zh-min",
    "zh-min-nan",
];

for (const tag of regularGrandfatheredWithExtLang) {
    assert.throws(RangeError, () => new Intl.Locale(tag));
}

// Add variants, extensions, and privateuse subtags to regular grandfathered
// language tags and ensure it produces the "expected" result.
const extras = [
    "fonipa",
    "a-not-assigned",
    "u-attr",
    "u-co",
    "u-co-phonebk",
    "x-private",
];

for (const {tag, canonical} of regularGrandfathered) {
    const priv = "-x-0";
    const tagMax = new Intl.Locale(canonical + priv).maximize().toString().slice(0, -priv.length);
    const tagMin = new Intl.Locale(canonical + priv).minimize().toString().slice(0, -priv.length);

    for (const extra of extras) {
        const loc = new Intl.Locale(tag + "-" + extra);

        let canonicalWithExtra = canonical + "-" + extra;
        let canonicalMax = tagMax + "-" + extra;
        let canonicalMin = tagMin + "-" + extra;

        // Ensure the added variant subtag is correctly sorted in the canonical tag.
        if (/^[a-z0-9]{5,8}|[0-9][a-z0-9]{3}$/i.test(extra)) {
            const sorted = s => s.replace(/(-([a-z0-9]{5,8}|[0-9][a-z0-9]{3}))+$/i,
                                          m => m.split("-").sort().join("-"));
            canonicalWithExtra = sorted(canonicalWithExtra);
            canonicalMax = sorted(canonicalMax);
            canonicalMin = sorted(canonicalMin);
        }

        // Adding extra subtags to grandfathered tags can have "interesting" results. Take for
        // example "art-lojban" when "fonipa" is added, so we get "art-lojban-fonipa". The first
        // step when canonicalising the language tag is to bring it in 'canonical syntax', that
        // means among other things sorting variants in alphabetical order. So "art-lojban-fonipa"
        // is transformed to "art-fonipa-lojban", because "fonipa" is sorted before "lojban". And
        // only after that has happened, we replace aliases with their preferred form.
        //
        // Now the usual problems arise when doing silly things like adding subtags to
        // grandfathered subtags, nobody, neither RFC 5646 nor UTS 35, provides a clear description
        // what needs to happen next.
        //
        // From <http://unicode.org/reports/tr35/#Language_Tag_to_Locale_Identifier>:
        // 
        // > A valid [BCP47] language tag can be converted to a valid Unicode BCP 47 locale 
        // > identifier according to Annex C. LocaleId Canonicalization
        // 
        // From <http://unicode.org/reports/tr35/#LocaleId_Canonicalization>
        // > The languageAlias, scriptAlias, territoryAlias, and variantAlias elements are used
        // > as rules to transform an input source localeId. The first step is to transform the
        // > languageId portion of the localeId.
        //
        // For regular grandfathered tags, "lojban", "gaulish", "guoyu", "hakka", and "xiang" will
        // therefore be considered as the "variant" subtag and be replaced by rules in languageAlias.
        //
        // Not all language tag processor will pass this test, for example because they don't order
        // variant subtags in alphabetical order or they're too eager when detecting grandfathered
        // tags. For example "zh-hakka-hakka" is accepted in some language tag processors, because
        // the language tag starts with a prefix which matches a grandfathered tag, and that prefix
        // is then canonicalised to "hak" and the second "hakka" is simply appended to it, so the
        // resulting tag is "hak-hakka". This is clearly wrong as far as ECMA-402 compliance is
        // concerned, because language tags are parsed and validated before any canonicalisation
        // happens. And during the validation step an error should be emitted, because the input
        // "zh-hakka-hakka" contains two identical variant subtags.
        //
        // From <https://tc39.es/ecma402/#sec-isstructurallyvalidlanguagetag>:
        //
        // > does not include duplicate variant subtags
        //
        // So, if your implementation fails this assertion, but you still like to test the rest of
        // this file, a pull request to split this file seems the way to go!
        assert.sameValue(loc.toString(), canonicalWithExtra);

        assert.sameValue(loc.maximize().toString(), canonicalMax);
        assert.sameValue(loc.maximize().maximize().toString(), canonicalMax);

        assert.sameValue(loc.minimize().toString(), canonicalMin);
        assert.sameValue(loc.minimize().minimize().toString(), canonicalMin);

        assert.sameValue(loc.maximize().minimize().toString(), canonicalMin);
        assert.sameValue(loc.minimize().maximize().toString(), canonicalMax);
    }
}
