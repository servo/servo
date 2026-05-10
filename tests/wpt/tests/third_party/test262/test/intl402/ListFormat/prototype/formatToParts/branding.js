// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.ListFormat.prototype.formatToParts
description: >
    Verifies the branding check for the "formatToParts" function of the ListFormat prototype object.
info: |
    Intl.ListFormat.prototype.formatToParts([ list ])

    2. If Type(lf) is not Object, throw a TypeError exception.
    3. If lf does not have an [[InitializedListFormat]] internal slot, throw a TypeError exception.
features: [Intl.ListFormat]
---*/

const formatToParts = Intl.ListFormat.prototype.formatToParts;

assert.sameValue(typeof formatToParts, "function");
assert.throws(TypeError, () => formatToParts.call(undefined), "undefined");
assert.throws(TypeError, () => formatToParts.call(null), "null");
assert.throws(TypeError, () => formatToParts.call(true), "true");
assert.throws(TypeError, () => formatToParts.call(""), "empty string");
assert.throws(TypeError, () => formatToParts.call(Symbol()), "symbol");
assert.throws(TypeError, () => formatToParts.call(1), "1");
assert.throws(TypeError, () => formatToParts.call({}), "plain object");
assert.throws(TypeError, () => formatToParts.call(Intl.ListFormat), "Intl.ListFormat");
assert.throws(TypeError, () => formatToParts.call(Intl.ListFormat.prototype), "Intl.ListFormat.prototype");
