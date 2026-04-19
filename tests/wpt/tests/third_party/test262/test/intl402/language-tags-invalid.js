// Copyright 2012 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 6.2.2_c
description: >
    Tests that language tags with invalid subtag sequences are not
    accepted.
author: Norbert Lindenberg
includes: [testIntl.js]
---*/

var invalidLanguageTags = getInvalidLanguageTags();

testWithIntlConstructors(function (Constructor) {
    invalidLanguageTags.forEach(function (tag) {
        // this must throw an exception for an invalid language tag
        assert.throws(RangeError, function() {
            var obj = new Constructor([tag]);
        }, "Invalid language tag " + tag + " was not rejected.");
    });
});
