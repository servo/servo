// Copyright (C) 2020 Rick Waldron.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-ecmascript-standard-built-in-objects
description: Object.seal '`length` property'
info: |
  ECMAScript Standard Built-in Objects

  Every built-in function object, including constructors, has a "length" property whose value is an integer. Unless otherwise specified, this value is equal to the number of required parameters shown in the subclause headings for the function description. Optional parameters and rest parameters are not included in the parameter count.

  Unless otherwise specified, the "length" property of a built-in function object has the attributes { [[Writable]]: false, [[Enumerable]]: false, [[Configurable]]: true }.

includes: [propertyHelper.js]
---*/

verifyProperty(Object.seal, "length", {
  value: 1,
  writable: false,
  enumerable: false,
  configurable: true,
});
