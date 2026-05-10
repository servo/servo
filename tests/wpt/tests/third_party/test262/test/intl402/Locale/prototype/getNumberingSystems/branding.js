// Copyright 2023 Google Inc.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.Locale.prototype.getNumberingSystems
description: Verifies the branding check for the "getNumberingSystems" function of the Locale prototype object.
info: |
    Intl.Locale.prototype.getNumberingSystems ()

    2. Perform ? RequireInternalSlot(loc, [[InitializedLocale]]).

features: [Intl.Locale,Intl.Locale-info]
---*/

const getNumberingSystems = Intl.Locale.prototype.getNumberingSystems;

assert.sameValue(typeof getNumberingSystems, "function");

assert.throws(TypeError, () => getNumberingSystems.call(undefined), "undefined");
assert.throws(TypeError, () => getNumberingSystems.call(null), "null");
assert.throws(TypeError, () => getNumberingSystems.call(true), "true");
assert.throws(TypeError, () => getNumberingSystems.call(""), "empty string");
assert.throws(TypeError, () => getNumberingSystems.call(Symbol()), "symbol");
assert.throws(TypeError, () => getNumberingSystems.call(1), "1");
assert.throws(TypeError, () => getNumberingSystems.call({}), "plain object");
assert.throws(TypeError, () => getNumberingSystems.call(Intl.Locale), "Intl.Locale");
assert.throws(TypeError, () => getNumberingSystems.call(Intl.Locale.prototype), "Intl.Locale.prototype");
