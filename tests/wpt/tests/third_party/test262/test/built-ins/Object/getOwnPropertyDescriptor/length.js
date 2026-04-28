// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.getownpropertydescriptor
description: >
  Object.getOwnPropertyDescriptor.length is 2.
info: |
  Object.getOwnPropertyDescriptor ( O, P )

  ECMAScript Standard Built-in Objects

  Every built-in function object, including constructors, has a "length" property whose
  value is an integer. Unless otherwise specified, this value is equal to the largest
  number of named arguments shown in the subclause headings for the function description.

  Unless otherwise specified, the "length" property of a built-in function object has
  the attributes { [[Writable]]: false, [[Enumerable]]: false, [[Configurable]]: true }.
includes: [propertyHelper.js]
---*/

verifyProperty(Object.getOwnPropertyDescriptor, "length", {
  value: 2,
  writable: false,
  enumerable: false,
  configurable: true,
});
