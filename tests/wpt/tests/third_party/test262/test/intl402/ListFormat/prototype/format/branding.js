// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.ListFormat.prototype.format
description: >
    Verifies the branding check for the "format" function of the ListFormat prototype object.
info: |
    Intl.ListFormat.prototype.format ([ list ])

    2. If Type(lf) is not Object, throw a TypeError exception.
    3. If lf does not have an [[InitializedListFormat]] internal slot, throw a TypeError exception.
features: [Intl.ListFormat]
---*/

const format = Intl.ListFormat.prototype.format;

assert.sameValue(typeof format, "function");

assert.throws(TypeError, () => format.call(undefined), "undefined");
assert.throws(TypeError, () => format.call(null), "null");
assert.throws(TypeError, () => format.call(true), "true");
assert.throws(TypeError, () => format.call(""), "empty string");
assert.throws(TypeError, () => format.call(Symbol()), "symbol");
assert.throws(TypeError, () => format.call(1), "1");
assert.throws(TypeError, () => format.call({}), "plain object");
assert.throws(TypeError, () => format.call(Intl.ListFormat), "Intl.ListFormat");
assert.throws(TypeError, () => format.call(Intl.ListFormat.prototype), "Intl.ListFormat.prototype");
