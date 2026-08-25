// Copyright 2012 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 9.2.8_4
description: >
    Tests that the array returned by SupportedLocales is extensible,
    writable and configurable.
author: Norbert Lindenberg
includes: [testIntl.js]
---*/

function testNormalProperty(obj, property) {
    var desc = Object.getOwnPropertyDescriptor(obj, property);
    assert.sameValue(desc.writable, true, "Property " + property + " of object returned by SupportedLocales is not writable.");
    assert.sameValue(desc.configurable, true, "Property " + property + " of object returned by SupportedLocales is not configurable.");
}

function testLengthProperty(obj, property) {
    var desc = Object.getOwnPropertyDescriptor(obj, property);
    assert.sameValue(desc.writable, true, "Property " + property + " of object returned by SupportedLocales is not writable.");
    assert.sameValue(desc.configurable, false, "Property " + property + " of object returned by SupportedLocales is configurable.");
}

testWithIntlConstructors(function (Constructor) {
    var defaultLocale = new Constructor().resolvedOptions().locale;
    var supported = Constructor.supportedLocalesOf([defaultLocale]);
    assert(Object.isExtensible(supported), "Object returned by SupportedLocales is not extensible.");
    for (var i = 0; i < supported.length; i++) {
        testNormalProperty(supported, i);
    }
    testLengthProperty(supported, "length");
});
