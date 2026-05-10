// Copyright 2012 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 9.1_a
description: Tests that default locale is available.
author: Norbert Lindenberg
includes: [testIntl.js]
---*/

testWithIntlConstructors(function (Constructor) {
    var defaultLocale = new Constructor().resolvedOptions().locale;
    var supportedLocales = Constructor.supportedLocalesOf([defaultLocale]);
    assert.notSameValue(supportedLocales.indexOf(defaultLocale), -1, "Default locale is not reported as available.");
});
