// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-426
description: >
    ES5 Attributes - success to update [[Get]] attribute of accessor
    property ([[Get]] is undefined, [[Set]] is undefined,
    [[Enumerable]] is true, [[Configurable]] is true) to different
    value
---*/

var obj = {};
var getFunc = function() {
  return 1001;
};

Object.defineProperty(obj, "prop", {
  get: undefined,
  set: undefined,
  enumerable: true,
  configurable: true
});

var result1 = typeof obj.prop === "undefined";
var desc1 = Object.getOwnPropertyDescriptor(obj, "prop");

Object.defineProperty(obj, "prop", {
  get: getFunc
});

var result2 = obj.prop === 1001;
var desc2 = Object.getOwnPropertyDescriptor(obj, "prop");

assert(result1, 'result1 !== true');
assert(result2, 'result2 !== true');
assert.sameValue(typeof desc1.get, "undefined", 'typeof desc1.get');
assert.sameValue(desc2.get, getFunc, 'desc2.get');
