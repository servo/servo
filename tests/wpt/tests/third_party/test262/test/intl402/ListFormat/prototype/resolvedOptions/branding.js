// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.ListFormat.prototype.resolvedOptions
description: Verifies the branding check for the "resolvedOptions" function of the ListFormat prototype object.
info: |
    Intl.ListFormat.prototype.resolvedOptions()

    2. If Type(pr) is not Object, throw a TypeError exception.
    3. If pr does not have an [[InitializedListFormat]] internal slot, throw a TypeError exception.
features: [Intl.ListFormat]
---*/

const resolvedOptions = Intl.ListFormat.prototype.resolvedOptions;

assert.sameValue(typeof resolvedOptions, "function");
assert.throws(TypeError, () => resolvedOptions.call(undefined), "undefined");
assert.throws(TypeError, () => resolvedOptions.call(null), "null");
assert.throws(TypeError, () => resolvedOptions.call(true), "true");
assert.throws(TypeError, () => resolvedOptions.call(""), "empty string");
assert.throws(TypeError, () => resolvedOptions.call(Symbol()), "symbol");
assert.throws(TypeError, () => resolvedOptions.call(1), "1");
assert.throws(TypeError, () => resolvedOptions.call({}), "plain object");
assert.throws(TypeError, () => resolvedOptions.call(Intl.ListFormat), "Intl.ListFormat");
assert.throws(TypeError, () => resolvedOptions.call(Intl.ListFormat.prototype), "Intl.ListFormat.prototype");
