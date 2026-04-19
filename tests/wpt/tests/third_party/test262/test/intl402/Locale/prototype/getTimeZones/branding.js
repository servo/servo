// Copyright 2023 Google Inc.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.Locale.prototype.getTimeZones
description: Verifies the branding check for the "getTimeZones" function of the Locale prototype object.
info: |
    Intl.Locale.prototype.getTimeZones ()

    2. Perform ? RequireInternalSlot(loc, [[InitializedLocale]]).

features: [Intl.Locale,Intl.Locale-info]
---*/

const getTimeZones = Intl.Locale.prototype.getTimeZones;

assert.sameValue(typeof getTimeZones, "function");

assert.throws(TypeError, () => getTimeZones.call(undefined), "undefined");
assert.throws(TypeError, () => getTimeZones.call(null), "null");
assert.throws(TypeError, () => getTimeZones.call(true), "true");
assert.throws(TypeError, () => getTimeZones.call(""), "empty string");
assert.throws(TypeError, () => getTimeZones.call(Symbol()), "symbol");
assert.throws(TypeError, () => getTimeZones.call(1), "1");
assert.throws(TypeError, () => getTimeZones.call({}), "plain object");
assert.throws(TypeError, () => getTimeZones.call(Intl.Locale), "Intl.Locale");
assert.throws(TypeError, () => getTimeZones.call(Intl.Locale.prototype), "Intl.Locale.prototype");
