// Copyright (C) 2009 the Sputnik authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-object.prototype.hasownproperty
description: >
  Object.prototype.hasOwnProperty.length is 1.
info: |
  Object.prototype.hasOwnProperty ( V )

  ECMAScript Standard Built-in Objects

  Every built-in function object, including constructors, has a "length"
  property whose value is an integer. Unless otherwise specified, this
  value is equal to the largest number of named arguments shown in the
  subclause headings for the function description.

  Unless otherwise specified, the "length" property of a built-in Function
  object has the attributes { [[Writable]]: false, [[Enumerable]]: false,
  [[Configurable]]: true }.
includes: [propertyHelper.js]
---*/

verifyProperty(Object.prototype.hasOwnProperty, "length", {
  value: 1,
  writable: false,
  enumerable: false,
  configurable: true,
});
