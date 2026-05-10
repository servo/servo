// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-403
description: >
    ES5 Attributes - Successfully add a property to an object when the
    object's prototype has a property with same name and [[Writable]]
    attribute is set to true (Array instance)
---*/

Object.defineProperty(Array.prototype, "prop", {
  value: 1001,
  writable: true,
  enumerable: true,
  configurable: true
});
var arrObj = [];
arrObj.prop = 1002;

assert(arrObj.hasOwnProperty("prop"), 'arrObj.hasOwnProperty("prop") !== true');
assert.sameValue(arrObj.prop, 1002, 'arrObj.prop');
