// Copyright 2012 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 9.2.1_1
description: >
    Tests that canonicalization of locale lists treats undefined and
    empty lists the same.
author: Norbert Lindenberg
includes: [testIntl.js]
---*/

testWithIntlConstructors(function (Constructor) {
    var supportedForUndefined = Constructor.supportedLocalesOf(undefined);
    var supportedForEmptyList = Constructor.supportedLocalesOf([]);
    assert.sameValue(supportedForUndefined.length, supportedForEmptyList.length, "Supported locales differ between undefined and empty list input.");
    // we don't compare the elements because length should be 0 - let's just verify that
    assert.sameValue(supportedForUndefined.length, 0, "Internal test error: Assumption about length being 0 is invalid.");
});
