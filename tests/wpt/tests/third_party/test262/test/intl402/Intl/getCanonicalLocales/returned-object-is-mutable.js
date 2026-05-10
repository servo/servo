// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-intl.getcanonicallocales
description: >
  Tests that the value returned by getCanonicalLocales is a mutable array.
info: |
  8.2.1 Intl.getCanonicalLocales (locales)
  1. Let ll be ? CanonicalizeLocaleList(locales).
  2. Return CreateArrayFromList(ll).
includes: [propertyHelper.js]
---*/

var locales = ['en-US', 'fr'];
var result = Intl.getCanonicalLocales(locales);

verifyProperty(result, 0, {
  value: 'en-US',
  writable: true,
  enumerable: true,
  configurable: true,
});

result = Intl.getCanonicalLocales(locales);

verifyProperty(result, 1, {
  value: 'fr',
  writable: true,
  enumerable: true,
  configurable: true,
});

result = Intl.getCanonicalLocales(locales);

verifyProperty(result, "length", {
  value: 2,
  writable: true,
  enumerable: false,
  configurable: false,
});

result.length = 42;
assert.sameValue(result.length, 42);

assert.throws(RangeError, function() {
  result.length = "Leo";
}, "a non-numeric value can't be set to result.length");
