// Copyright 2023 Google Inc.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.Locale.prototype.getCollations
description: Verifies the branding check for the "getCollations" function of the Locale prototype object.
info: |
    Intl.Locale.prototype.getCollations ()

    2. Perform ? RequireInternalSlot(loc, [[InitializedLocale]]).

features: [Intl.Locale,Intl.Locale-info]
---*/

const getCollations = Intl.Locale.prototype.getCollations;

assert.sameValue(typeof getCollations, "function");

assert.throws(TypeError, () => getCollations.call(undefined), "undefined");
assert.throws(TypeError, () => getCollations.call(null), "null");
assert.throws(TypeError, () => getCollations.call(true), "true");
assert.throws(TypeError, () => getCollations.call(""), "empty string");
assert.throws(TypeError, () => getCollations.call(Symbol()), "symbol");
assert.throws(TypeError, () => getCollations.call(1), "1");
assert.throws(TypeError, () => getCollations.call({}), "plain object");
assert.throws(TypeError, () => getCollations.call(Intl.Locale), "Intl.Locale");
assert.throws(TypeError, () => getCollations.call(Intl.Locale.prototype), "Intl.Locale.prototype");
