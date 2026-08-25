// Copyright 2012 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 11.1.1_6
description: >
    Tests that the behavior of a Record is not affected by
    adversarial  changes to Object.prototype.
author: Norbert Lindenberg
includes: [testIntl.js]
---*/

taintProperties(["localeMatcher"]);

var locale = new Intl.NumberFormat(undefined, {localeMatcher: "lookup"}).resolvedOptions().locale;
assert(isCanonicalizedStructurallyValidLanguageTag(locale), "NumberFormat returns invalid locale " + locale + ".");
