// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-219-1
description: >
    Object.defineProperty - 'Attributes' is an Array object that uses
    Object's [[Get]] method to access the 'get' property of prototype
    object (8.10.5 step 7.a)
---*/

var obj = {};

Array.prototype.get = function() {
  return "arrayGetProperty";
};
var arrObj = [];

Object.defineProperty(obj, "property", arrObj);

assert.sameValue(obj.property, "arrayGetProperty", 'obj.property');
