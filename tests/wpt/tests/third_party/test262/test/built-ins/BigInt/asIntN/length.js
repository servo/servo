// Copyright (C) 2017 Josh Wolfe. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-bigint.asintn
description: BigInt.asIntN.length descriptor
info: |
  BigInt.asIntN ( bits, bigint )

  17 ECMAScript Standard Built-in Objects

includes: [propertyHelper.js]
features: [BigInt]
---*/

verifyProperty(BigInt.asIntN, "length", {
  value: 2,
  enumerable: false,
  writable: false,
  configurable: true
});
