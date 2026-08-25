/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Redefining an array's |length| property when redefining the |length| property on a proxy with an array as target
info: bugzilla.mozilla.org/show_bug.cgi?id=905947
esid: pending
---*/

var arr = [];
var p = new Proxy(arr, {});

// Redefining non-configurable length should throw a TypeError
assert.throws(TypeError, function () { Object.defineProperty(p, "length", { value: 17, configurable: true }); });

// Same here.
assert.throws(TypeError, function () { Object.defineProperty(p, "length", { value: 42, enumerable: true }); });

// Check the property went unchanged.
var pd = Object.getOwnPropertyDescriptor(p, "length");
assert.sameValue(pd.value, 0);
assert.sameValue(pd.writable, true);
assert.sameValue(pd.enumerable, false);
assert.sameValue(pd.configurable, false);

var ad = Object.getOwnPropertyDescriptor(arr, "length");
assert.sameValue(ad.value, 0);
assert.sameValue(ad.writable, true);
assert.sameValue(ad.enumerable, false);
assert.sameValue(ad.configurable, false);
