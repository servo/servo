// Copyright (C) 2017 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-typedarray.prototype.constructor
description: BigUint64Array.prototype.constructor property descriptor
info: |
  22.2.6.2 TypedArray.prototype.constructor

  The initial value of a TypedArray.prototype.constructor is the
  corresponding %TypedArray% intrinsic object.

  17 ECMAScript Standard Built-in Objects
includes: [propertyHelper.js]
features: [BigInt]
---*/

verifyProperty(BigUint64Array.prototype, "constructor", {
  value: BigUint64Array,
  writable: true,
  enumerable: false,
  configurable: true
});
