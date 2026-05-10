// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-225-1
description: >
    Object.defineProperty - 'Attributes' is a RegExp object that uses
    Object's [[Get]] method to access the 'get' property of prototype
    object (8.10.5 step 7.a)
---*/

var obj = {};

RegExp.prototype.get = function() {
  return "regExpGetProperty";
};
var regObj = new RegExp();


Object.defineProperty(obj, "property", regObj);

assert.sameValue(obj.property, "regExpGetProperty", 'obj.property');
