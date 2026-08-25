// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.RelativeTimeFormat
description: >
    Verifies the NewTarget check for Intl.RelativeTimeFormat.
info: |
    Intl.RelativeTimeFormat ([ locales [ , options ]])

    1. If NewTarget is undefined, throw a TypeError exception.
features: [Intl.RelativeTimeFormat]
---*/

assert.sameValue(typeof Intl.RelativeTimeFormat, "function");

assert.throws(TypeError, function() {
  Intl.RelativeTimeFormat();
});

assert.throws(TypeError, function() {
  Intl.RelativeTimeFormat("en");
});

assert.throws(TypeError, function() {
  Intl.RelativeTimeFormat("not-valid-tag");
});
