// Copyright (C) 2017 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-typedarray-constructors
description: BigInt64Array.length property descriptor
info: |
  The TypedArray Constructors

  The length property of the TypedArray constructor function is 3.

  17 ECMAScript Standard Built-in Objects

  ...

  Unless otherwise specified, the length property of a built-in function
  object has the attributes { [[Writable]]: false, [[Enumerable]]: false,
  [[Configurable]]: true }.
includes: [propertyHelper.js]
features: [BigInt, TypedArray]
---*/

verifyProperty(BigInt64Array, "length", {
  value: 3,
  writable: false,
  enumerable: false,
  configurable: true
});
