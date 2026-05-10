// Copyright 2016 Mozilla Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
esid: sec-intl.getcanonicallocales
description: Tests that the value returned by getCanonicalLocales is an Array.
info: |
  8.2.1 Intl.getCanonicalLocales (locales)
  1. Let ll be ? CanonicalizeLocaleList(locales).
  2. Return CreateArrayFromList(ll).
---*/

var locales = ['en-US'];
var result = Intl.getCanonicalLocales(['en-US']);

assert.sameValue(Object.getPrototypeOf(result), Array.prototype, 'prototype is Array.prototype');
assert.sameValue(result.constructor, Array);

assert.notSameValue(result, locales, "result is a new array instance");
assert.sameValue(result.length, 1, "result.length");
assert(result.hasOwnProperty("0"), "result an own property `0`");
assert.sameValue(result[0], "en-US", "result[0]");
