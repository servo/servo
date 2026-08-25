// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-429
description: >
    ES5 Attributes - success to update [[Configurable]] attribute of
    accessor property ([[Get]] is undefined, [[Set]] is undefined,
    [[Enumerable]] is true, [[Configurable]] is true) to different
    value
includes: [propertyHelper.js]
---*/

var obj = {};

Object.defineProperty(obj, "prop", {
  get: undefined,
  set: undefined,
  enumerable: true,
  configurable: true
});
var desc1 = Object.getOwnPropertyDescriptor(obj, "prop");

Object.defineProperty(obj, "prop", {
  configurable: false
});

assert.sameValue(desc1.configurable, true);

verifyProperty(obj, "prop", {
  configurable: false,
});
