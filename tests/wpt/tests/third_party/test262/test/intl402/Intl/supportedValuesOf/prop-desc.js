// Copyright (C) 2021 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.supportedvaluesof
description: >
  Intl.supportedValuesOf property attributes.
info: |
  Intl.supportedValuesOf ( key )

  18 ECMAScript Standard Built-in Objects:
    Every other data property described in clauses 19 through 28 and in Annex B.2
    has the attributes { [[Writable]]: true, [[Enumerable]]: false,
    [[Configurable]]: true } unless otherwise specified.
includes: [propertyHelper.js]
features: [Intl-enumeration]
---*/

verifyProperty(Intl, "supportedValuesOf", {
  writable: true,
  enumerable: false,
  configurable: true,
});
