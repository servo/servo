// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-349
description: >
    ES5 Attributes - property ([[Writable]] is true, [[Enumerable]] is
    false, [[Configurable]] is false) is undeletable
includes: [propertyHelper.js]
---*/

var obj = {};

Object.defineProperty(obj, "prop", {
  value: 2010,
  writable: true,
  enumerable: false,
  configurable: false
});

verifyProperty(obj, "prop", {
  value: 2010,
  configurable: false,
});
