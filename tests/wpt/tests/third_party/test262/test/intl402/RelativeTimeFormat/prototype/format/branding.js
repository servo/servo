// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.RelativeTimeFormat.prototype.format
description: Verifies the branding check for the "format" function of the RelativeTimeFormat prototype object.
info: |
    Intl.RelativeTimeFormat.prototype.format( value, unit )

    2. If Type(relativeTimeFormat) is not Object or relativeTimeFormat does not have an [[InitializedRelativeTimeFormat]] internal slot whose value is true, throw a TypeError exception.
features: [Intl.RelativeTimeFormat]
---*/

const format = Intl.RelativeTimeFormat.prototype.format;

assert.sameValue(typeof format, "function");

assert.throws(TypeError, () => format.call(undefined), "undefined");
assert.throws(TypeError, () => format.call(null), "null");
assert.throws(TypeError, () => format.call(true), "true");
assert.throws(TypeError, () => format.call(""), "empty string");
assert.throws(TypeError, () => format.call(Symbol()), "symbol");
assert.throws(TypeError, () => format.call(1), "1");
assert.throws(TypeError, () => format.call({}), "plain object");
assert.throws(TypeError, () => format.call(Intl.RelativeTimeFormat), "Intl.RelativeTimeFormat");
assert.throws(TypeError, () => format.call(Intl.RelativeTimeFormat.prototype), "Intl.RelativeTimeFormat.prototype");
