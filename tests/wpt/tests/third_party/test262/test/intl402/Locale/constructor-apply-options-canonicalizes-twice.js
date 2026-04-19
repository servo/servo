// Copyright 2020 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-apply-options-to-tag
description: >
    ApplyOptionsToTag canonicalises the language tag two times.
info: |
    10.1.1 ApplyOptionsToTag( tag, options )

    ...
    9. Set tag to CanonicalizeUnicodeLocaleId(tag).
    10. If language is not undefined,
        ...
        b. Set tag to tag with the substring corresponding to the unicode_language_subtag
           production of the unicode_language_id replaced by the string language.
    ...
    13. Return CanonicalizeUnicodeLocaleId(tag).
features: [Intl.Locale]
---*/

// ApplyOptionsToTag canonicalises the locale identifier before applying the
// options. That means "und-Armn-SU" is first canonicalised to "und-Armn-AM",
// then the language is changed to "ru". If "ru" were applied first, the result
// would be "ru-Armn-RU" instead.
assert.sameValue(new Intl.Locale("und-Armn-SU", {language: "ru"}).toString(), "ru-Armn-AM");
