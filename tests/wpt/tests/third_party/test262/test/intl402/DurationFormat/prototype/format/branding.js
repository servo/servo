// Copyright 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DurationFormat.prototype.format
description: Verifies the branding check for the "format" function of the DurationFormat prototype object.
features: [Intl.DurationFormat]
---*/

const format = Intl.DurationFormat.prototype.format;

assert.sameValue(typeof format, "function");

assert.throws(TypeError, () => format.call(undefined, { years : 2 }), "undefined");
assert.throws(TypeError, () => format.call(null, { years : 2 }), "null");
assert.throws(TypeError, () => format.call(true, { years : 2 }), "true");
assert.throws(TypeError, () => format.call("", { years : 2 }), "empty string");
assert.throws(TypeError, () => format.call(Symbol(), { years : 2 }), "symbol");
assert.throws(TypeError, () => format.call(1, { years : 2 }), "1");
assert.throws(TypeError, () => format.call({}, { years : 2 }), "plain object");
assert.throws(TypeError, () => format.call(Intl.DurationFormat, { years : 2 } ), "Intl.DurationFormat");
assert.throws(TypeError, () => format.call(Intl.DurationFormat.prototype,  { years : 2 }), "Intl.DurationFormat.prototype");
