// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.Segmenter.prototype.resolvedOptions
description: Verifies that the return value of Intl.Segmenter.prototype.resolvedOptions() is not cached.
info: |
    Intl.Segmenter.prototype.resolvedOptions ()

    3. Let options be ! ObjectCreate(%ObjectPrototype%).
features: [Intl.Segmenter]
---*/

const s = new Intl.Segmenter("en-us");
const options1 = s.resolvedOptions();
const options2 = s.resolvedOptions();
assert.notSameValue(options1, options2, "Should create a new object each time.");
