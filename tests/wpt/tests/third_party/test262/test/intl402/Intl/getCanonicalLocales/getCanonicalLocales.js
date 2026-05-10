// Copyright 2016 Mozilla Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
esid: sec-intl.getcanonicallocales
description: Property type and descriptor. 
info: |
  8.2.1 Intl.getCanonicalLocales (locales)
  1. Let ll be ? CanonicalizeLocaleList(locales).
  2. Return CreateArrayFromList(ll).
includes: [propertyHelper.js]
---*/

assert.sameValue(
  typeof Intl.getCanonicalLocales,
  'function',
  '`typeof Intl.getCanonicalLocales` is `function`'
);

verifyProperty(Intl, "getCanonicalLocales", {
  writable: true,
  enumerable: false,
  configurable: true,
});
