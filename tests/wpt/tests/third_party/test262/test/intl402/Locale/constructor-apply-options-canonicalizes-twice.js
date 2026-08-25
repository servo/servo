// Copyright 2020 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.Locale
description: >
    Intl.Locale constructor canonicalises the language tag two times.
info: |
    ...
    12. NOTE: Because LanguageId canonicalization can alter _tag_ in arbitrary
        ways according to Alias Rules from supplementalMetadata.xml, it is
        necessary to perform such canonicalization before applying overrides
        from _options_.
    13. Set tag to CanonicalizeUnicodeLocaleId(tag).
    14. Set tag to ? UpdateLanguageId(tag, options).
    ...

    UpdateLanguageId ( _tag_, _options_ )

    ...
    11. Let _newTag_ be _language_.
    12. If _script_ is not *undefined*, set _newTag_ to the string-concatenation
        of _newTag_, *"-"*, and _script_.
    13. If _region_ is not *undefined*, set _newTag_ to the string-concatenation
        of _newTag_, *"-"*, and _region_.
    14. If _variants_ is not *undefined*, set _newTag_ to the string-
        concatenation of _newTag_, *"-"*, and _variants_.
    15. Set _newTag_ to the string-concatenation of _newTag_ and
        _allExtensions_.
    16. Return _newTag_.

    MakeLocaleRecord ( _tag_, _options_, _localeExtensionKeys_ )

    ...
    6. If _attributes_ is not empty or _keywords_ is not empty, then
      a. Set _result_.[[locale]] to InsertUnicodeExtensionAndCanonicalize(
         _locale_, _attributes_, _keywords_).
    7. Else,
      a. Set _result_.[[locale]] to CanonicalizeUnicodeLocaleId(_locale_).
    8. Return _result_.
features: [Intl.Locale]
---*/

// Intl.Locale constructor canonicalises the locale identifier before applying
// the options. That means "und-Armn-SU" is first canonicalised to
// "und-Armn-AM", then the language is changed to "ru". If "ru" were applied
// first, the result would be "ru-Armn-RU" instead.
assert.sameValue(new Intl.Locale("und-Armn-SU", {language: "ru"}).toString(), "ru-Armn-AM");
