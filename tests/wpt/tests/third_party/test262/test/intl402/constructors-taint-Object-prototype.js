// Copyright 2012 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 9.2.3_5
description: >
    Tests that the behavior of a Record is not affected by
    adversarial  changes to Object.prototype.
author: Norbert Lindenberg
includes: [testIntl.js]
---*/

taintProperties(["locale", "extension", "extensionIndex"]);

testWithIntlConstructors(function (Constructor) {
    var locale = new Constructor(undefined, {localeMatcher: "lookup"}).resolvedOptions().locale;
    assert(isCanonicalizedStructurallyValidLanguageTag(locale), "Constructor returns invalid locale " + locale + ".");
});
