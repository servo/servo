// Copyright 2023 Google Inc.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.Locale.prototype.getWeekInfo
description: Verifies the branding check for the "getWeekInfo" function of the Locale prototype object.
info: |
    Intl.Locale.prototype.getWeekInfo ()

    2. Perform ? RequireInternalSlot(loc, [[InitializedLocale]]).

features: [Intl.Locale,Intl.Locale-info]
---*/

const getWeekInfo = Intl.Locale.prototype.getWeekInfo;

assert.sameValue(typeof getWeekInfo, "function");

assert.throws(TypeError, () => getWeekInfo.call(undefined), "undefined");
assert.throws(TypeError, () => getWeekInfo.call(null), "null");
assert.throws(TypeError, () => getWeekInfo.call(true), "true");
assert.throws(TypeError, () => getWeekInfo.call(""), "empty string");
assert.throws(TypeError, () => getWeekInfo.call(Symbol()), "symbol");
assert.throws(TypeError, () => getWeekInfo.call(1), "1");
assert.throws(TypeError, () => getWeekInfo.call({}), "plain object");
assert.throws(TypeError, () => getWeekInfo.call(Intl.Locale), "Intl.Locale");
assert.throws(TypeError, () => getWeekInfo.call(Intl.Locale.prototype), "Intl.Locale.prototype");
