// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-368
description: >
    ES5 Attributes - property ([[Writable]] is false, [[Enumerable]]
    is false, [[Configurable]] is true) is unwritable
includes: [propertyHelper.js]
---*/

var obj = {};

Object.defineProperty(obj, "prop", {
  value: 2010,
  writable: false,
  enumerable: false,
  configurable: true
});

verifyProperty(obj, "prop", {
  value: 2010,
  writable: false,
});
