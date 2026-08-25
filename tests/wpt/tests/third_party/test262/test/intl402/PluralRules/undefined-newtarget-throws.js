// Copyright 2016 Mozilla Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
esid: sec-Intl.PluralRules
description: Tests that PluralRules throws when called as a function
author: Zibi Braniecki
includes: [testIntl.js]
---*/

assert.throws(TypeError, function() {
  Intl.PluralRules();
}, "Intl.PluralRules throws when called as a function");

assert.throws(TypeError, function() {
  Intl.PluralRules.call(undefined);
}, "Intl.PluralRules throws when called as a function with |undefined| as this-value");

testWithIntlConstructors(function (Constructor) {
    var obj = new Constructor();

    assert.throws(TypeError, function() {
        Intl.PluralRules.call(obj)
    }, "Intl.PluralRules throws when called as a function with an Intl-object as this-value");
});
