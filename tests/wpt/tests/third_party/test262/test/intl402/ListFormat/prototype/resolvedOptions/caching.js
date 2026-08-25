// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.ListFormat.prototype.resolvedOptions
description: Verifies that the return value of Intl.ListFormat.prototype.resolvedOptions() is not cached.
info: |
    Intl.ListFormat.prototype.resolvedOptions ()

    4. Let options be ! ObjectCreate(%ObjectPrototype%).
features: [Intl.ListFormat]
---*/

const lf = new Intl.ListFormat("en-us");
const options1 = lf.resolvedOptions();
const options2 = lf.resolvedOptions();
assert.notSameValue(options1, options2, "Should create a new object each time.");
