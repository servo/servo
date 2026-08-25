// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-227
description: >
    Object.defineProperty - 'Attributes' is an Error object that uses
    Object's [[Get]] method to access the 'get' property (8.10.5 step
    7.a)
---*/

var obj = {};

var errObj = new Error();

errObj.get = function() {
  return "errorGetProperty";
};

Object.defineProperty(obj, "property", errObj);

assert.sameValue(obj.property, "errorGetProperty", 'obj.property');
