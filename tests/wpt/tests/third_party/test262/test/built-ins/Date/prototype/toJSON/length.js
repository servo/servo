// Copyright (C) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.tojson
description: >
  Date.prototype.toJSON.length is 1.
info: |
  Date.prototype.toJSON ( key )

  ECMAScript Standard Built-in Objects

  Unless otherwise specified, the length property of a built-in Function
  object has the attributes { [[Writable]]: false, [[Enumerable]]: false,
  [[Configurable]]: true }.
includes: [propertyHelper.js]
---*/

verifyProperty(Date.prototype.toJSON, 'length', {
  value: 1,
  writable: false,
  enumerable: false,
  configurable: true,
});
