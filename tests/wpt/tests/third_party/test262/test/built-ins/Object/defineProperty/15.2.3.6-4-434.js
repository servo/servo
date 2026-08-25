// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-434
description: >
    ES5 Attributes - property ([[Get]] is undefined, [[Set]] is
    undefined, [[Enumerable]] is true, [[Configurable]] is false) is
    undeletable
includes: [propertyHelper.js]
---*/

var obj = {};

Object.defineProperty(obj, "prop", {
  get: undefined,
  set: undefined,
  enumerable: true,
  configurable: false
});

verifyProperty(obj, "prop", {
  configurable: false,
});
