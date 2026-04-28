// Copyright 2012 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 9.2.2
description: >
    Tests that locales that are reported by resolvedOptions  are also
    reported by supportedLocalesOf.
author: Norbert Lindenberg
includes: [testIntl.js]
---*/

testWithIntlConstructors(function (Constructor) {
    // this test should work equally for both matching algorithms
    ["lookup", "best fit"].forEach(function (matcher) {
        var info = getLocaleSupportInfo(Constructor, {localeMatcher: matcher});
        var supportedByConstructor = info.supported.concat(info.byFallback);
        var supported = Constructor.supportedLocalesOf(supportedByConstructor,
            {localeMatcher: matcher});
        // we could check the length first, but it's probably more interesting which locales are missing
        var i = 0;
        var limit = Math.min(supportedByConstructor.length, supported.length);
        while (i < limit && supportedByConstructor[i] === supported[i]) {
            i++;
        }
        assert.sameValue(i < supportedByConstructor.length, false, "Locale " + supportedByConstructor[i] + " is returned by resolvedOptions but not by supportedLocalesOf.");
        assert.sameValue(i < supported.length, false, "Locale " + supported[i] + " is returned by supportedLocalesOf but not by resolvedOptions.");
    });
    
    // this test is only valid for lookup - best fit may find additional locales supported
    var info = getLocaleSupportInfo(Constructor, {localeMatcher: "lookup"});
    var unsupportedByConstructor = info.unsupported;
    var supported = Constructor.supportedLocalesOf(unsupportedByConstructor,
            {localeMatcher: "lookup"});
    assert.sameValue(supported.length > 0, false, "Locale " + supported[0] + " is returned by supportedLocalesOf but not by resolvedOptions.");
});
