// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.getcanonicallocales
description: >
  Intl.getCanonicalLocales property attributes.
info: |
  8.2.1 Intl.getCanonicalLocales (locales)

  17 ECMAScript Standard Built-in Objects:
    Every other data property described in clauses 18 through 26 and in
    Annex B.2 has the attributes { [[Writable]]: true, [[Enumerable]]: false,
    [[Configurable]]: true } unless otherwise specified.
includes: [propertyHelper.js]
---*/

verifyProperty(Intl, 'getCanonicalLocales', {
  writable: true,
  enumerable: false,
  configurable: true,
});
