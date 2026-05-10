// Copyright 2023 Google Inc.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.Locale.prototype.getCalendars
description: Verifies the branding check for the "getCalendars" function of the Locale prototype object.
info: |
    Intl.Locale.prototype.getCalendars ()

    2. Perform ? RequireInternalSlot(loc, [[InitializedLocale]]).

features: [Intl.Locale,Intl.Locale-info]
---*/

const getCalendars = Intl.Locale.prototype.getCalendars;

assert.sameValue(typeof getCalendars, "function");

assert.throws(TypeError, () => getCalendars.call(undefined), "undefined");
assert.throws(TypeError, () => getCalendars.call(null), "null");
assert.throws(TypeError, () => getCalendars.call(true), "true");
assert.throws(TypeError, () => getCalendars.call(""), "empty string");
assert.throws(TypeError, () => getCalendars.call(Symbol()), "symbol");
assert.throws(TypeError, () => getCalendars.call(1), "1");
assert.throws(TypeError, () => getCalendars.call({}), "plain object");
assert.throws(TypeError, () => getCalendars.call(Intl.Locale), "Intl.Locale");
assert.throws(TypeError, () => getCalendars.call(Intl.Locale.prototype), "Intl.Locale.prototype");
