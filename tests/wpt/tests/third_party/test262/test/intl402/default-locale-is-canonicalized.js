// Copyright 2012 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 6.2.4
esid: sec-defaultlocale
description: >
    Tests that the default locale is a String value representing the
    structurally valid and canonicalized BCP 47 language tag, and that
    it does not contain a Unicode locale extension sequence.
info: |
    DefaultLocale ( )

    The returned String value represents the well-formed and canonicalized
    language tag for the host environment's current locale. It must not contain
    a Unicode locale extension sequence.

    ResolveLocale ( availableLocales, requestedLocales, options,
                    relevantExtensionKeys, localeData )

    ...
    4. If match is undefined, set match to the Record { [[locale]]:
       DefaultLocale(), [[extension]]: empty }.
author: Norbert Lindenberg
includes: [testIntl.js]
---*/

testWithIntlConstructors(function (Constructor) {
    var defaultLocale = new Constructor().resolvedOptions().locale;
    assert(isCanonicalizedStructurallyValidLanguageTag(defaultLocale), "Default locale \"" + defaultLocale + "\" is not canonicalized and structurally valid language tag.");
    assert.sameValue(defaultLocale.indexOf("-u-"), -1, "Default locale \"" + defaultLocale + "\" contains a Unicode locale extension sequence.");
});
