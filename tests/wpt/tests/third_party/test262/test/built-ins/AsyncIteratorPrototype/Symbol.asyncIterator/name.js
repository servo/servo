// Copyright (C) 2018 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-asynciteratorprototype-asynciterator
description: Descriptor for `name` property
info: |
  %AsyncIteratorPrototype% [ @@asyncIterator ] ( )
  ...
  The value of the name property of this function is "[Symbol.asyncIterator]".

  ECMAScript Standard Built-in Objects
  ...
  Every built-in Function object, including constructors, that is not
  identified as an anonymous function has a name property whose value is a
  String. Unless otherwise specified, this value is the name that is given to
  the function in this specification.
  ...
  Unless otherwise specified, the name property of a built-in Function
  object, if it exists, has the attributes { [[Writable]]: false,
  [[Enumerable]]: false, [[Configurable]]: true }.
features: [Symbol.asyncIterator, async-iteration]
includes: [propertyHelper.js]
---*/

async function* generator() {}
var AsyncIteratorPrototype = Object.getPrototypeOf(Object.getPrototypeOf(generator.prototype))

verifyProperty(AsyncIteratorPrototype[Symbol.asyncIterator], "name", {
  value: '[Symbol.asyncIterator]',
  enumerable: false,
  writable: false,
  configurable: true,
});
