// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.RelativeTimeFormat.prototype.formatToParts
description: Verifies the branding check for the "formatToParts" function of the RelativeTimeFormat prototype object.
info: |
    Intl.RelativeTimeFormat.prototype.formatToParts( value, unit )

    2. If Type(relativeTimeFormat) is not Object or relativeTimeFormat does not have an [[InitializedRelativeTimeFormat]] internal slot whose value is true, throw a TypeError exception.
features: [Intl.RelativeTimeFormat]
---*/

const formatToParts = Intl.RelativeTimeFormat.prototype.formatToParts;

assert.sameValue(typeof formatToParts, "function");

assert.throws(TypeError, () => formatToParts.call(undefined), "undefined");
assert.throws(TypeError, () => formatToParts.call(null), "null");
assert.throws(TypeError, () => formatToParts.call(true), "true");
assert.throws(TypeError, () => formatToParts.call(""), "empty string");
assert.throws(TypeError, () => formatToParts.call(Symbol()), "symbol");
assert.throws(TypeError, () => formatToParts.call(1), "1");
assert.throws(TypeError, () => formatToParts.call({}), "plain object");
assert.throws(TypeError, () => formatToParts.call(Intl.RelativeTimeFormat), "Intl.RelativeTimeFormat");
assert.throws(TypeError, () => formatToParts.call(Intl.RelativeTimeFormat.prototype), "Intl.RelativeTimeFormat.prototype");
