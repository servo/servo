// Copyright (C) 2012 Ecma International. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-json.stringify
description: >
  JSON.stringify.length is 3.
info: |
  JSON.stringify ( value [ , replacer [ , space ] ] )

  The "length" property of the stringify function is 3.

  ECMAScript Standard Built-in Objects

  Unless otherwise specified, the length property of a built-in Function
  object has the attributes { [[Writable]]: false, [[Enumerable]]: false,
  [[Configurable]]: true }.
includes: [propertyHelper.js]
---*/

verifyProperty(JSON.stringify, 'length', {
  value: 3,
  writable: false,
  enumerable: false,
  configurable: true,
});
