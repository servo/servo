// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%
description: >
  TypedArray has a 'name' property whose value is "TypedArray".
info: |
  22.2.2 Properties of the %TypedArray% Intrinsic Object

  Besides a length property whose value is 3 and a name property whose value is
  "TypedArray", %TypedArray% has the following properties:
  ...

  ES6 section 17: Unless otherwise specified, the name property of a built-in
  Function object, if it exists, has the attributes { [[Writable]]: false,
  [[Enumerable]]: false, [[Configurable]]: true }.
includes: [propertyHelper.js, testTypedArray.js]
features: [TypedArray]
---*/

verifyProperty(TypedArray, "name", {
  value: "TypedArray",
  writable: false,
  enumerable: false,
  configurable: true
});
