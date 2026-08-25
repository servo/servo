// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-374
description: >
    ES5 Attributes - success to update the data property ([[Writable]]
    is false, [[Enumerable]] is false, [[Configurable]] is true) to an
    accessor property
---*/

var obj = {};

Object.defineProperty(obj, "prop", {
  value: 2010,
  writable: false,
  enumerable: false,
  configurable: true
});
var desc1 = Object.getOwnPropertyDescriptor(obj, "prop");

function getFunc() {
  return 20;
}
Object.defineProperty(obj, "prop", {
  get: getFunc
});
var desc2 = Object.getOwnPropertyDescriptor(obj, "prop");

assert(desc1.hasOwnProperty("value"), 'desc1.hasOwnProperty("value") !== true');
assert.sameValue(desc2.get, getFunc, 'desc2.get');
