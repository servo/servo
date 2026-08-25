// Copyright (C) 2017 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-properties-of-the-typedarray-constructors
description: BigUint64Array prototype internal slot
info: |
  22.2.5 Properties of the TypedArray Constructors

  The value of the [[Prototype]] internal slot of each TypedArray
  constructor is the %TypedArray% intrinsic object.
includes: [testTypedArray.js]
features: [BigInt, TypedArray]
---*/

assert.sameValue(Object.getPrototypeOf(BigUint64Array), TypedArray);
