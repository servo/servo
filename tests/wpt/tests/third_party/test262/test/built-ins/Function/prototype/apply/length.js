// Copyright 2009 the Sputnik authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-function.prototype.apply
description: >
  Function.prototype.apply.length is 2.
info: |
  ECMAScript Standard Built-in Objects
  ...
  Every built-in Function object, including constructors, has a length
  property whose value is an integer. Unless otherwise specified, this value
  is equal to the largest number of named arguments shown in the subclause
  headings for the function description, including optional parameters.
  ...
  Unless otherwise specified, the length property of a built-in Function
  object has the attributes { [[Writable]]: false, [[Enumerable]]: false,
  [[Configurable]]: true }.
includes: [propertyHelper.js]
---*/

verifyProperty(Function.prototype.apply, 'length', {
  value: 2,
  writable: false,
  enumerable: false,
  configurable: true,
});
