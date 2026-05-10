// Copyright 2012 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 9.2.1_4
description: >
    Tests that non-objects are converted to objects before
    canonicalization.
author: Norbert Lindenberg
includes: [testIntl.js]
---*/

testWithIntlConstructors(function (Constructor) {
    // undefined is handled separately
    
    // null should result in a TypeError
    assert.throws(TypeError, function() {
        var supportedForNull = Constructor.supportedLocalesOf(null);
    }, "Null as locale list was not rejected.");
    
    // let's use an empty list for comparison
    var supportedForEmptyList = Constructor.supportedLocalesOf([]);
    // we don't compare the elements because length should be 0 - let's just verify that
    assert.sameValue(supportedForEmptyList.length, 0, "Internal test error: Assumption about length being 0 is invalid.");

    // most non-objects will be interpreted as empty lists because a missing length property is interpreted as 0
    var supportedForNumber = Constructor.supportedLocalesOf(5);
    assert.sameValue(supportedForNumber.length, supportedForEmptyList.length, "Supported locales differ between numeric and empty list input.");
    var supportedForBoolean = Constructor.supportedLocalesOf(true);
    assert.sameValue(supportedForBoolean.length, supportedForEmptyList.length, "Supported locales differ between boolean and empty list input.");
});
