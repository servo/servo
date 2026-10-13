// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.includes
description: >
  The "name" property of Iterator.prototype.includes
info: |
  ECMAScript Standard Built-in Objects

  Every built-in Function object, including constructors, that is not
  identified as an anonymous function has a name property whose value is a
  String. Unless otherwise specified, this value is the name that is given to
  the function in this specification.

  Unless otherwise specified, the name property of a built-in Function
  object, if it exists, has the attributes { [[Writable]]: false,
  [[Enumerable]]: false, [[Configurable]]: true }.

includes: [propertyHelper.js]
features: [iterator-includes]
---*/

verifyProperty(Iterator.prototype.includes, 'name', {
  value: 'includes',
  writable: false,
  enumerable: false,
  configurable: true,
});
