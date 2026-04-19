// Copyright 2023 Google Inc.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.Locale.prototype.getHourCycles
description: Verifies the branding check for the "getHourCycles" function of the Locale prototype object.
info: |
    Intl.Locale.prototype.getHourCycles ()

    2. Perform ? RequireInternalSlot(loc, [[InitializedLocale]]).

features: [Intl.Locale,Intl.Locale-info]
---*/

const getHourCycles = Intl.Locale.prototype.getHourCycles;

assert.sameValue(typeof getHourCycles, "function");

assert.throws(TypeError, () => getHourCycles.call(undefined), "undefined");
assert.throws(TypeError, () => getHourCycles.call(null), "null");
assert.throws(TypeError, () => getHourCycles.call(true), "true");
assert.throws(TypeError, () => getHourCycles.call(""), "empty string");
assert.throws(TypeError, () => getHourCycles.call(Symbol()), "symbol");
assert.throws(TypeError, () => getHourCycles.call(1), "1");
assert.throws(TypeError, () => getHourCycles.call({}), "plain object");
assert.throws(TypeError, () => getHourCycles.call(Intl.Locale), "Intl.Locale");
assert.throws(TypeError, () => getHourCycles.call(Intl.Locale.prototype), "Intl.Locale.prototype");
