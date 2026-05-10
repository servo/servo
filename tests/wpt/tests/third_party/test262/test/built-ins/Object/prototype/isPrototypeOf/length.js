// Copyright (C) 2009 the Sputnik authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.prototype.isprototypeof
description: >
  Object.prototype.isPrototypeOf.length is 1.
info: |
  Object.prototype.isPrototypeOf ( V )

  ECMAScript Standard Built-in Objects

  Unless otherwise specified, the length property of a built-in Function
  object has the attributes { [[Writable]]: false, [[Enumerable]]: false,
  [[Configurable]]: true }.
includes: [propertyHelper.js]
---*/

verifyProperty(Object.prototype.isPrototypeOf, "length", {
  value: 1,
  writable: false,
  enumerable: false,
  configurable: true,
});
