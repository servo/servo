// Copyright 2012 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 9.2.6_4_b
description: >
    Tests that Unicode locale extension sequences do not affect
    whether a locale is considered supported, but are reported back.
author: Norbert Lindenberg
includes: [testIntl.js]
---*/

testWithIntlConstructors(function (Constructor) {

    // this test should work equally for both matching algorithms
    ["lookup", "best fit"].forEach(function (matcher) {
        var opt = {localeMatcher: matcher};
        var info = getLocaleSupportInfo(Constructor, opt);
        var allLocales = info.supported.concat(info.byFallback, info.unsupported);
        allLocales.forEach(function (locale) {
            var validExtension = "-u-co-phonebk-nu-latn";
            var invalidExtension = "-u-nu-invalid";
            var supported1 = Constructor.supportedLocalesOf([locale], opt);
            var supported2 = Constructor.supportedLocalesOf([locale + validExtension], opt);
            var supported3 = Constructor.supportedLocalesOf([locale + invalidExtension], opt);
            if (supported1.length === 1) {
                assert.sameValue(supported2.length, 1, "#1.1: Presence of Unicode locale extension sequence affects whether locale " + locale + " is considered supported with matcher " + matcher + ".");
                assert.sameValue(supported3.length, 1, "#1.2: Presence of Unicode locale extension sequence affects whether locale " + locale + " is considered supported with matcher " + matcher + ".");
                assert.sameValue(supported2[0], locale + validExtension, "#2.1: Unicode locale extension sequence is not correctly returned for locale " + locale + " with matcher " + matcher + ".");
                assert.sameValue(supported3[0], locale + invalidExtension, "#2.2: Unicode locale extension sequence is not correctly returned for locale " + locale + " with matcher " + matcher + ".");
            } else {
                assert.sameValue(supported2.length, 0, "#3.1: Presence of Unicode locale extension sequence affects whether locale " + locale + " is considered supported with matcher " + matcher + ".");
                assert.sameValue(supported3.length, 0, "#3.2: Presence of Unicode locale extension sequence affects whether locale " + locale + " is considered supported with matcher " + matcher + ".");
            }
        });
    });
});
