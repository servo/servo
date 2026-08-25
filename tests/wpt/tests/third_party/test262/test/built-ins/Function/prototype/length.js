// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-properties-of-the-function-prototype-object
description: >
  Function.prototype.length is 0.
info: |
  Properties of the Function Prototype Object

  The Function prototype object:

  [...]
  * has a "length" property whose value is 0.

  ECMAScript Standard Built-in Objects

  Unless otherwise specified, the "length" property of a built-in function object has
  the attributes { [[Writable]]: false, [[Enumerable]]: false, [[Configurable]]: true }.
includes: [propertyHelper.js]
---*/

verifyProperty(Function.prototype, "length", {
  value: 0,
  writable: false,
  enumerable: false,
  configurable: true,
});
