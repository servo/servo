// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.at
description: >
  TypedArray.prototype.at.length value and descriptor.
info: |
  %TypedArray%.prototype.at( index )

  17 ECMAScript Standard Built-in Objects

includes: [propertyHelper.js, testTypedArray.js]
features: [TypedArray.prototype.at]
---*/
assert.sameValue(
  typeof TypedArray.prototype.at,
  'function',
  'The value of `typeof TypedArray.prototype.at` is "function"'
);

verifyProperty(TypedArray.prototype.at, "length", {
  value: 1,
  writable: false,
  enumerable: false,
  configurable: true
});
