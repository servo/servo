// Copyright 2023 Google Inc.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.Locale.prototype.getTextInfo
description: Verifies the branding check for the "getTextInfo" function of the Locale prototype object.
info: |
    Intl.Locale.prototype.getTextInfo ()

    2. Perform ? RequireInternalSlot(loc, [[InitializedLocale]]).

features: [Intl.Locale,Intl.Locale-info]
---*/

const getTextInfo = Intl.Locale.prototype.getTextInfo;

assert.sameValue(typeof getTextInfo, "function");

assert.throws(TypeError, () => getTextInfo.call(undefined), "undefined");
assert.throws(TypeError, () => getTextInfo.call(null), "null");
assert.throws(TypeError, () => getTextInfo.call(true), "true");
assert.throws(TypeError, () => getTextInfo.call(""), "empty string");
assert.throws(TypeError, () => getTextInfo.call(Symbol()), "symbol");
assert.throws(TypeError, () => getTextInfo.call(1), "1");
assert.throws(TypeError, () => getTextInfo.call({}), "plain object");
assert.throws(TypeError, () => getTextInfo.call(Intl.Locale), "Intl.Locale");
assert.throws(TypeError, () => getTextInfo.call(Intl.Locale.prototype), "Intl.Locale.prototype");
