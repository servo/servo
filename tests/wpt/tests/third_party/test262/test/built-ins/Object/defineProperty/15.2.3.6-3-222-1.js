// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-222-1
description: >
    Object.defineProperty - 'Attributes' is a Number object that uses
    Object's [[Get]] method to access the 'get' property of prototype
    object (8.10.5 step 7.a)
---*/

var obj = {};

Number.prototype.get = function() {
  return "numberGetProperty";
};
var numObj = new Number(-2);

Object.defineProperty(obj, "property", numObj);

assert.sameValue(obj.property, "numberGetProperty", 'obj.property');
