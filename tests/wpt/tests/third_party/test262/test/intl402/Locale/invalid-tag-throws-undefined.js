// Copyright 2018 André Bargull; Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.locale
description: >
    Verifies the type check on the tag argument to Intl.Locale.
info: |
    Intl.Locale( tag [, options] )

    7. If Type(tag) is not String or Object, throw a TypeError exception.
features: [Intl.Locale]
---*/

assert.sameValue(typeof Intl.Locale, "function");

assert.throws(TypeError, function() {
  new Intl.Locale();
}, "(empty) is an invalid tag value");

assert.throws(TypeError, function() {
  new Intl.Locale(undefined)
}, "undefined is an invalid tag value");
