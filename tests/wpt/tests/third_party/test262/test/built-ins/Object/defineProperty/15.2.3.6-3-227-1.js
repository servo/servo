// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-227-1
description: >
    Object.defineProperty - 'Attributes' is an Error object that uses
    Object's [[Get]] method to access the 'get' property of prototype
    object (8.10.5 step 7.a)
---*/

var obj = {};

Error.prototype.get = function() {
  return "errorGetProperty";
};
var errObj = new Error();

Object.defineProperty(obj, "property", errObj);

assert.sameValue(obj.property, "errorGetProperty", 'obj.property');
