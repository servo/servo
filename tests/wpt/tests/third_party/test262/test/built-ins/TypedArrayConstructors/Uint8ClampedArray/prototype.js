// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-typedarray.prototype
description: >
  The initial value of Uint8ClampedArray.prototype is the Uint8ClampedArray prototype object.
info: |
  The initial value of TypedArray.prototype is the corresponding TypedArray prototype intrinsic object (22.2.6).

  This property has the attributes { [[Writable]]: false, [[Enumerable]]: false, [[Configurable]]: false }.
includes: [propertyHelper.js]
features: [TypedArray]
---*/

assert.sameValue(Uint8ClampedArray.prototype, Object.getPrototypeOf(new Uint8ClampedArray(0)));

verifyProperty(Uint8ClampedArray, "prototype", {
  writable: false,
  enumerable: false,
  configurable: false,
});
