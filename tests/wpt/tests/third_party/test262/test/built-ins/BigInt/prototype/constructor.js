// Copyright (C) 2017 Robin Templeton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-bigint.prototype.constructor
description: BigInt.prototype.constructor property descriptor
info: |
  BigInt.prototype.constructor

  The initial value of BigInt.prototype.constructor is the intrinsic
  object %BigInt%.

  The BigInt Constructor

  The BigInt constructor is the %BigInt% intrinsic object and the
  initial value of the BigInt property of the global object. When BigInt
  is called as a function, it performs a type conversion.

  17 ECMAScript Standard Built-in Objects
includes: [propertyHelper.js]
features: [BigInt]
---*/

verifyProperty(BigInt.prototype, "constructor", {
  value: BigInt,
  writable: true,
  enumerable: false,
  configurable: true
});
