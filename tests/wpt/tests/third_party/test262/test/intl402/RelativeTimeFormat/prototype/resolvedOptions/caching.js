// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.RelativeTimeFormat.prototype.resolvedOptions
description: Verifies that the return value of Intl.RelativeTimeFormat.prototype.resolvedOptions() is not cached.
info: |
    Intl.RelativeTimeFormat.prototype.resolvedOptions ()

    4. Let options be ! ObjectCreate(%ObjectPrototype%).
features: [Intl.RelativeTimeFormat]
---*/

const rtf = new Intl.RelativeTimeFormat("en-us");
const options1 = rtf.resolvedOptions();
const options2 = rtf.resolvedOptions();
assert.notSameValue(options1, options2, "Should create a new object each time.");
