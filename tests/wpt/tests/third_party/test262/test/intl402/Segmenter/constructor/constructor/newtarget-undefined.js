// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.Segmenter
description: Verifies the NewTarget check for Intl.Segmenter.
info: |
    Intl.Segmenter ([ locales [ , options ]])

    1. If NewTarget is undefined, throw a TypeError exception.
features: [Intl.Segmenter]
---*/

assert.sameValue(typeof Intl.Segmenter, "function");

assert.throws(TypeError, function() {
  Intl.Segmenter();
});

assert.throws(TypeError, function() {
  Intl.Segmenter("en");
});

assert.throws(TypeError, function() {
  Intl.Segmenter("not-valid-tag");
});
