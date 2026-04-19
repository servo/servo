// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.RelativeTimeFormat.prototype.format
description: Checks the handling of invalid value arguments to Intl.RelativeTimeFormat.prototype.format().
info: |
    Intl.RelativeTimeFormat.prototype.format( value, unit )

    3. Let value be ? ToNumber(value).

features: [Intl.RelativeTimeFormat]
---*/

const rtf = new Intl.RelativeTimeFormat("en-US");

assert.sameValue(typeof rtf.format, "function", "format should be supported");

const symbol = Symbol();
assert.throws(TypeError, () => rtf.format(symbol, "second"));
