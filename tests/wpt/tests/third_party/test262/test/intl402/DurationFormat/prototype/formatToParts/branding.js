// Copyright 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DurationFormat.prototype.formatToParts
description: Verifies the branding check for the "formatToParts" function of the DurationFormat prototype object.
features: [Intl.DurationFormat]
---*/

const formatToParts = Intl.DurationFormat.prototype.formatToParts;

assert.sameValue(typeof formatToParts, "function");

assert.throws(TypeError, () => formatToParts.call(undefined, { years : 2 }), "undefined");
assert.throws(TypeError, () => formatToParts.call(null, { years : 2 }), "null");
assert.throws(TypeError, () => formatToParts.call(true, { years : 2 }), "true");
assert.throws(TypeError, () => formatToParts.call("", { years : 2 }), "empty string");
assert.throws(TypeError, () => formatToParts.call(Symbol(), { years : 2 }), "symbol");
assert.throws(TypeError, () => formatToParts.call(1, { years : 2 }), "1");
assert.throws(TypeError, () => formatToParts.call({}, { years : 2 }), "plain object");
assert.throws(TypeError, () => formatToParts.call(Intl.DurationFormat, { years : 2 } ), "Intl.DurationFormat");
assert.throws(TypeError, () => formatToParts.call(Intl.DurationFormat.prototype,  { years : 2 }), "Intl.DurationFormat.prototype");
