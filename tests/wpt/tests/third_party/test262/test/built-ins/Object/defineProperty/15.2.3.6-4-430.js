// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-430
description: >
    ES5 Attributes - success to update the accessor property ([[Get]]
    is undefined, [[Set]] is undefined, [[Enumerable]] is true,
    [[Configurable]] is true) to a data property
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
  value: 1001
});
var desc2 = Object.getOwnPropertyDescriptor(obj, "prop");

assert(desc1.hasOwnProperty("get"), 'desc1.hasOwnProperty("get") !== true');
assert(desc2.hasOwnProperty("value"), 'desc2.hasOwnProperty("value") !== true');
