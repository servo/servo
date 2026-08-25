// Copyright (C) 2017 Robin Templeton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: The property descriptor BigInt.prototype
esid: sec-bigint.prototype
info: |
  This property has the attributes { [[Writable]]: false, [[Enumerable]]: false,
  [[Configurable]]: false }.
features: [BigInt]
includes: [propertyHelper.js]
---*/

verifyProperty(BigInt, "prototype", {
  writable: false,
  enumerable: false,
  configurable: false
});
