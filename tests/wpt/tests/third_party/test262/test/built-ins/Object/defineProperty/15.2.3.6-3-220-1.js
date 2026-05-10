// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-220-1
description: >
    Object.defineProperty - 'Attributes' is a String object that uses
    Object's [[Get]] method to access the 'get' property of prototype
    object (8.10.5 step 7.a)
---*/

var obj = {};

String.prototype.get = function() {
  return "stringGetProperty";
};
var strObj = new String();

Object.defineProperty(obj, "property", strObj);

assert.sameValue(obj.property, "stringGetProperty", 'obj.property');
