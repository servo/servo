// Copyright (C) 2021 Jamie Kyle.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.hasown
description: >
  Object.hasOwn.length is 2.
info: |
  Object.hasOwn ( _O_, _P_ )

  ECMAScript Standard Built-in Objects

  Every built-in function object, including constructors, has a "length"
  property whose value is an integer. Unless otherwise specified, this
  value is equal to the largest number of named arguments shown in the
  subclause headings for the function description.

  Unless otherwise specified, the "length" property of a built-in Function
  object has the attributes { [[Writable]]: false, [[Enumerable]]: false,
  [[Configurable]]: true }.
includes: [propertyHelper.js]
author: Jamie Kyle
features: [Object.hasOwn]
---*/

verifyProperty(Object.hasOwn, "length", {
  value: 2,
  writable: false,
  enumerable: false,
  configurable: true,
});
