// Copyright 2012 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 9.1_b
description: >
    Tests that appropriate fallback locales are provided for
    supported locales.
author: Norbert Lindenberg
includes: [testIntl.js]
features: [Array.prototype.includes]
---*/

testWithIntlConstructors(function (Constructor) {
    // The test is only valid under "lookup" localeMatcher
    var info = getLocaleSupportInfo(Constructor, {localeMatcher: "lookup"});
    for (var locale of info.supported) {
        var match = /^([a-z]{2,3})(-[A-Z][a-z]{3})?(-(?:[A-Z]{2}|[0-9]{3}))?$/.exec(locale);
        assert.notSameValue(match, null, "Locale " + locale + " is supported, but can't be parsed.")

        var [language, script, region] = match.slice(1);

        if (script !== undefined) {
            var fallback = language + script;
            assert(info.supported.includes(fallback) || info.byFallback.includes(fallback),
                   "Locale " + locale + " is supported, but fallback " + fallback + " isn't.");
        }

        if (region !== undefined) {
            var fallback = language + region;
            assert(info.supported.includes(fallback) || info.byFallback.includes(fallback),
                   "Locale " + locale + " is supported, but fallback " + fallback + " isn't.");
        }
    }
});
