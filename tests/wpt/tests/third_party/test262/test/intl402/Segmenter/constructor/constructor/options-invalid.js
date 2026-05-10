// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.Segmenter
description: Checks handling of a null options argument to the Segmenter constructor.
info: |
    Intl.Segmenter ([ locales [ , options ]])

    5. Else
        a. Let options be ? ToObject(options).
features: [Intl.Segmenter]
---*/

assert.sameValue(typeof Intl.Segmenter, "function");

assert.throws(TypeError, function() { new Intl.Segmenter([], null) })
