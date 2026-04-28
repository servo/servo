// Copyright (C) 2018 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-asynciteratorprototype-asynciterator
description: Length of AsyncIteratorPrototype[ @@asyncIterator ]
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
features: [Symbol.asyncIterator, async-iteration]
includes: [propertyHelper.js]
---*/

async function* generator() {}
var AsyncIteratorPrototype = Object.getPrototypeOf(Object.getPrototypeOf(generator.prototype))

verifyProperty(AsyncIteratorPrototype[Symbol.asyncIterator], "length", {
  value: 0,
  enumerable: false,
  writable: false,
  configurable: true,
});
