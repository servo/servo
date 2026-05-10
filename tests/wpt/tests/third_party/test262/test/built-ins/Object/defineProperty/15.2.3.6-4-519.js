// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-519
description: >
    ES5 Attributes - success to update [[Configurable]] attribute of
    accessor property ([[Get]] is a Function, [[Set]] is undefined,
    [[Enumerable]] is false, [[Configurable]] is true) to different
    value
includes: [propertyHelper.js]
---*/

var obj = {};

var getFunc = function() {
  return 1001;
};

Object.defineProperty(obj, "prop", {
  get: getFunc,
  set: undefined,
  enumerable: false,
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
