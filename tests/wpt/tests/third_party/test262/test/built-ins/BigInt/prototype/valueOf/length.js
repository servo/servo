// Copyright (C) 2017 Robin Templeton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-bigint.prototype.valueof
description: BigInt.prototype.valueOf.length property descriptor
info: |
  BigInt.prototype.valueOf ( )

  17 ECMAScript Standard Built-in Objects
includes: [propertyHelper.js]
features: [BigInt]
---*/

verifyProperty(BigInt.prototype.valueOf, "length", {
  value: 0,
  writable: false,
  enumerable: false,
  configurable: true
});
