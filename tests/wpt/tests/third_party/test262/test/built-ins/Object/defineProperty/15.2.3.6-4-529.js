// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-529
description: >
    ES5 Attributes - fail to update the accessor property ([[Get]] is
    a Function, [[Set]] is undefined, [[Enumerable]] is false,
    [[Configurable]] is false) to a data property
---*/

var obj = {};

var getFunc = function() {
  return 1001;
};

Object.defineProperty(obj, "prop", {
  get: getFunc,
  set: undefined,
  enumerable: false,
  configurable: false
});
var desc1 = Object.getOwnPropertyDescriptor(obj, "prop");
assert.throws(TypeError, function() {
  Object.defineProperty(obj, "prop", {
    value: 1001
  });
});
var desc2 = Object.getOwnPropertyDescriptor(obj, "prop");

assert(desc1.hasOwnProperty("get"), 'desc1.hasOwnProperty("get") !== true');
assert.sameValue(desc2.hasOwnProperty("value"), false, 'desc2.hasOwnProperty("value")');
