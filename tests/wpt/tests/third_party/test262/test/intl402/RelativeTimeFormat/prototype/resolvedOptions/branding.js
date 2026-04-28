// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.RelativeTimeFormat.prototype.resolvedOptions
description: Verifies the branding check for the "resolvedOptions" function of the RelativeTimeFormat prototype object.
info: |
    Intl.RelativeTimeFormat.prototype.resolvedOptions ()

    2. If Type(relativeTimeFormat) is not Object or relativeTimeFormat does not have an [[InitializedRelativeTimeFormat]] internal slot whose value is true, throw a TypeError exception.
features: [Intl.RelativeTimeFormat]
---*/

const resolvedOptions = Intl.RelativeTimeFormat.prototype.resolvedOptions;

assert.sameValue(typeof resolvedOptions, "function");

assert.throws(TypeError, () => resolvedOptions.call(undefined), "undefined");
assert.throws(TypeError, () => resolvedOptions.call(null), "null");
assert.throws(TypeError, () => resolvedOptions.call(true), "true");
assert.throws(TypeError, () => resolvedOptions.call(""), "empty string");
assert.throws(TypeError, () => resolvedOptions.call(Symbol()), "symbol");
assert.throws(TypeError, () => resolvedOptions.call(1), "1");
assert.throws(TypeError, () => resolvedOptions.call({}), "plain object");
assert.throws(TypeError, () => resolvedOptions.call(Intl.RelativeTimeFormat), "Intl.RelativeTimeFormat");
assert.throws(TypeError, () => resolvedOptions.call(Intl.RelativeTimeFormat.prototype), "Intl.RelativeTimeFormat.prototype");
