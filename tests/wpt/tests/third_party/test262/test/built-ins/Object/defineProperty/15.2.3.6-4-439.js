// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-439
description: >
    ES5 Attributes - fail to update the accessor property ([[Get]] is
    undefined, [[Set]] is undefined, [[Enumerable]] is true,
    [[Configurable]] is false) to a data property
---*/

var obj = {};

Object.defineProperty(obj, "prop", {
  get: undefined,
  set: undefined,
  enumerable: true,
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
