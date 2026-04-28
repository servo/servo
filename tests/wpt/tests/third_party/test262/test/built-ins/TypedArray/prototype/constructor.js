// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.constructor
description: >
  Initial state of the constructor property
info: |
  The initial value of %TypedArray%.prototype.constructor is the %TypedArray% intrinsic object.

  Per ES6 section 17, the method should exist on the %TypedArray% prototype, and it
  should be writable and configurable, but not enumerable.
includes: [propertyHelper.js, testTypedArray.js]
features: [TypedArray]
---*/

assert.sameValue(TypedArray.prototype.constructor, TypedArray);

verifyProperty(TypedArray.prototype, "constructor", {
  writable: true,
  enumerable: false,
  configurable: true,
});
