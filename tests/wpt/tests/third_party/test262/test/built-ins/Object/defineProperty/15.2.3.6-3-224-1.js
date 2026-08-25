// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-224-1
description: >
    Object.defineProperty - 'Attributes' is a Date object that uses
    Object's [[Get]] method to access the 'get' property of prototype
    object (8.10.5 step 7.a)
---*/

var obj = {};

Date.prototype.get = function() {
  return "dateGetProperty";
};
var dateObj = new Date();

Object.defineProperty(obj, "property", dateObj);

assert.sameValue(obj.property, "dateGetProperty", 'obj.property');
