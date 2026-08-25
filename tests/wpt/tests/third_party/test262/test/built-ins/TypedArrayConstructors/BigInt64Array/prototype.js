// Copyright (C) 2017 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-typedarray.prototype
description: BigInt64Array.prototype property descriptor
info: |
  22.2.5.2 TypedArray.prototype

  The initial value of TypedArray.prototype is the corresponding
  TypedArray prototype intrinsic object (22.2.6).

  This property has the attributes { [[Writable]]: false,
  [[Enumerable]]: false, [[Configurable]]: false }.
includes: [propertyHelper.js]
features: [BigInt]
---*/

verifyProperty(BigInt64Array, "prototype", {
  value: Object.getPrototypeOf(new BigInt64Array()),
  writable: false,
  enumerable: false,
  configurable: false
});
