// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-408
description: >
    ES5 Attributes - Successfully add a property to an object when the
    object's prototype has a property with same name and [[Writable]]
    attribute is set to true (Date instance)
---*/

Object.defineProperty(Date.prototype, "prop", {
  value: 1001,
  writable: true,
  enumerable: true,
  configurable: true
});
var dateObj = new Date();
dateObj.prop = 1002;

assert(dateObj.hasOwnProperty("prop"), 'dateObj.hasOwnProperty("prop") !== true');
assert.sameValue(dateObj.prop, 1002, 'dateObj.prop');
