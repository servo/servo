// Copyright 2016 Mozilla Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
esid: sec-properties-of-intl-pluralrules-prototype-object
description: >
    Tests that Intl.PluralRules.prototype is not an object that has been
    initialized as an Intl.PluralRules.
author: Zibi Braniecki
---*/

// test by calling a function that fails if "this" is not an object
// initialized as an Intl.PluralRules
assert.throws(TypeError, function() {
    Intl.PluralRules.prototype.select(0);
}, "Intl.PluralRules.prototype is not an object that has been initialized as an Intl.PluralRules");
