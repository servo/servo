// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype-@@iterator
description: >
  Initial state of the Symbol.iterator property
info: |
  The initial value of the @@iterator property is the same function object
  as the initial value of the %TypedArray%.prototype.values property.

  Per ES6 section 17, the method should exist on the %TypedArray% prototype, and it
  should be writable and configurable, but not enumerable.
includes: [propertyHelper.js, testTypedArray.js]
features: [Symbol.iterator]
---*/

assert.sameValue(TypedArray.prototype[Symbol.iterator], TypedArray.prototype.values);

verifyProperty(TypedArray.prototype, Symbol.iterator, {
  writable: true,
  enumerable: false,
  configurable: true,
});
