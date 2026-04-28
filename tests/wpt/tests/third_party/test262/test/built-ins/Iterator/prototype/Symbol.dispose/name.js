// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%iteratorprototype%-@@dispose
description: Descriptor for `name` property of %IteratorPrototype%[ @@dispose ]
info: |
  The value of the name property of this function is "[Symbol.dispose]".

  ES6 Section 17: ECMAScript Standard Built-in Objects

  Every built-in Function object, including constructors, that is not
  identified as an anonymous function has a name property whose value is a
  String. Unless otherwise specified, this value is the name that is given to
  the function in this specification.

  [...]

  Unless otherwise specified, the name property of a built-in Function
  object, if it exists, has the attributes { [[Writable]]: false,
  [[Enumerable]]: false, [[Configurable]]: true }.
features: [explicit-resource-management]
includes: [propertyHelper.js]
---*/
const IteratorPrototype = Object.getPrototypeOf(
  Object.getPrototypeOf([][Symbol.iterator]())
);

verifyProperty(IteratorPrototype[Symbol.dispose], 'name', {
  value: '[Symbol.dispose]',
  writable: false,
  enumerable: false,
  configurable: true,
});
