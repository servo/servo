// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.locale
description: >
    Verifies the NewTarget check for Intl.Locale.
info: |
    Intl.Locale( tag [, options] )

    1. If NewTarget is undefined, throw a TypeError exception.
features: [Intl.Locale]
---*/

assert.sameValue(typeof Intl.Locale, "function");

assert.throws(TypeError, function() {
  Intl.Locale();
}, 'Intl.Locale() throws TypeError');

assert.throws(TypeError, function() {
  Intl.Locale("en");
}, 'Intl.Locale("en") throws TypeError');
