// Copyright 2012 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 9.2.6_4_c
description: >
    Tests that LookupSupportedLocales includes the default locale  and
    doesn't include the "no linguistic content" locale.
author: Norbert Lindenberg
includes: [testIntl.js]
---*/

testWithIntlConstructors(function (Constructor) {
    // this test should work equally for both matching algorithms
    ["lookup", "best fit"].forEach(function (matcher) {
        var defaultLocale = new Constructor().resolvedOptions().locale;
        var noLinguisticContent = "zxx";
        var supported = Constructor.supportedLocalesOf([defaultLocale, noLinguisticContent],
            {localeMatcher: matcher});
        assert.notSameValue(supported.indexOf(defaultLocale), -1, "SupportedLocales didn't return default locale with matcher " + matcher + ".");
        assert.sameValue(supported.indexOf(noLinguisticContent), -1, "SupportedLocales returned the \"no linguistic content\" locale with matcher " + matcher + ".");
        assert.sameValue(supported.length > 1, false, "SupportedLocales returned stray locales: " + supported.join(", ") + " with matcher " + matcher + ".");
    });
});
