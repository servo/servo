// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%iteratorprototype%-@@dispose
description: Length of %IteratorPrototype%[ @@dispose ]
info: |
    %IteratorPrototype% [ @@dispose ] ( )

    ES6 Section 17:
    Every built-in Function object, including constructors, has a length
    property whose value is an integer. Unless otherwise specified, this value
    is equal to the largest number of named arguments shown in the subclause
    headings for the function description, including optional parameters.

    [...]

    Unless otherwise specified, the length property of a built-in Function
    object has the attributes { [[Writable]]: false, [[Enumerable]]: false,
    [[Configurable]]: true }.
features: [explicit-resource-management]
includes: [propertyHelper.js]
---*/
const IteratorPrototype = Object.getPrototypeOf(
  Object.getPrototypeOf([][Symbol.iterator]())
);

verifyProperty(IteratorPrototype[Symbol.dispose], 'length', {
  value: 0,
  writable: false,
  enumerable: false,
  configurable: true,
});
