// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.at
description: >
  %TypedArray%.prototype.at.name value and descriptor.
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

assert.sameValue(
  TypedArray.prototype.at.name, 'at',
  'The value of TypedArray.prototype.at.name is "at"'
);

verifyProperty(TypedArray.prototype.at, 'name', {
  enumerable: false,
  writable: false,
  configurable: true
});
