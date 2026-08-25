// Copyright (C) 2016 The V8 Project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-number.nan
description: >
  "NaN" property descriptor and value of Number
info: |
  20.1.2.10 Number.NaN

  The value of Number.NaN is NaN.

  This property has the attributes { [[Writable]]: false, [[Enumerable]]: false,
  [[Configurable]]: false }.
includes: [propertyHelper.js]
---*/

assert.sameValue(Number.NaN, NaN);

verifyProperty(Number, "NaN", {
  writable: false,
  enumerable: false,
  configurable: false,
});
