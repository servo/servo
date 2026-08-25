// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.ListFormat
description: >
    Verifies the NewTarget check for Intl.ListFormat.
info: |
    Intl.ListFormat ([ locales [ , options ]])

    1. If NewTarget is undefined, throw a TypeError exception.
features: [Intl.ListFormat]
---*/

assert.sameValue(typeof Intl.ListFormat, "function");

assert.throws(TypeError, function() {
  Intl.ListFormat();
});

assert.throws(TypeError, function() {
  Intl.ListFormat("en");
});

assert.throws(TypeError, function() {
  Intl.ListFormat("not-valid-tag");
});
