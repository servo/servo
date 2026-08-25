// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-363
description: >
    ES5 Attributes - property ([[Writable]] is false, [[Enumerable]]
    is true, [[Configurable]] is false) is undeletable
includes: [propertyHelper.js]
---*/

var obj = {};

Object.defineProperty(obj, "prop", {
  value: 2010,
  writable: false,
  enumerable: true,
  configurable: false
});

verifyProperty(obj, "prop", {
  configurable: false,
});
