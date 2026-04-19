// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.RelativeTimeFormat.prototype.formatToParts
description: Checks the handling of invalid value arguments to Intl.RelativeTimeFormat.prototype.formatToParts().
info: |
    Intl.RelativeTimeFormat.prototype.formatToParts( value, unit )

    3. Let value be ? ToNumber(value).

features: [Intl.RelativeTimeFormat]
---*/

const rtf = new Intl.RelativeTimeFormat("en-US");

assert.sameValue(typeof rtf.formatToParts, "function", "formatToParts should be supported");

const symbol = Symbol();
assert.throws(TypeError, () => rtf.formatToParts(symbol, "second"));
