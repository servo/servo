// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%
description: >
  TypedArray has a "length" property whose value is 0.
info: |
  %TypedArray% ( )

  The length property of the %TypedArray% constructor function is 0.

  17 ECMAScript Standard Built-in Objects

  ...

  Unless otherwise specified, the length property of a built-in function
  object has the attributes { [[Writable]]: false, [[Enumerable]]: false,
  [[Configurable]]: true }.
includes: [propertyHelper.js, testTypedArray.js]
features: [TypedArray]
---*/

verifyProperty(TypedArray, "length", {
  value: 0,
  writable: false,
  enumerable: false,
  configurable: true
});
