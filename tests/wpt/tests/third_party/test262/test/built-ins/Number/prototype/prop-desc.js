// Copyright (C) 2016 The V8 Project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-number.prototype
description: >
  "prototype" property of Number
info: |
  Number.prototype

  This property has the attributes { [[Writable]]: false, [[Enumerable]]: false,
  [[Configurable]]: false }.
includes: [propertyHelper.js]
---*/

verifyProperty(Number, "prototype", {
  writable: false,
  enumerable: false,
  configurable: false,
});
