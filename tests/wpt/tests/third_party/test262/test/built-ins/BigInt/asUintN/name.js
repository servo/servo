// Copyright (C) 2017 Josh Wolfe. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-bigint.asuintn
description: BigInt.asUintN.name descriptor
info: |
  BigInt.asUintN ( bits, bigint )

  17 ECMAScript Standard Built-in Objects

includes: [propertyHelper.js]
features: [BigInt]
---*/

verifyProperty(BigInt.asUintN, "name", {
  value: "asUintN",
  enumerable: false,
  writable: false,
  configurable: true
});
