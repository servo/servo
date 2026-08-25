// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.2.5.15
description: >
    "length" property of `unicode` accessor function
info: |
    ES6 section 17: Every built-in Function object, including constructors, has
    a length property whose value is an integer. Unless otherwise specified,
    this value is equal to the largest number of named arguments shown in the
    subclause headings for the function description, including optional
    parameters.

    [...]

    Unless otherwise specified, the length property of a built-in Function
    object has the attributes { [[Writable]]: false, [[Enumerable]]: false,
    [[Configurable]]: true }.
includes: [propertyHelper.js]
---*/

var getter = Object.getOwnPropertyDescriptor(RegExp.prototype, 'unicode').get;

verifyProperty(getter, "length", {
  value: 0,
  writable: false,
  enumerable: false,
  configurable: true
});
