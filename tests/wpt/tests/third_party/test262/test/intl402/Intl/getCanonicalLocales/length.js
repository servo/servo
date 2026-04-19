// Copyright 2016 Mozilla Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
esid: sec-intl.getcanonicallocales
description: Intl.getCanonicalLocales.length. 
info: |
  8.2.1 Intl.getCanonicalLocales (locales)
  1. Let ll be ? CanonicalizeLocaleList(locales).
  2. Return CreateArrayFromList(ll).
includes: [propertyHelper.js]
---*/

verifyProperty(Intl.getCanonicalLocales, "length", {
  value: 1,
  writable: false,
  enumerable: false,
  configurable: true,
});
