// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-225
description: >
    Object.defineProperty - 'Attributes' is a RegExp object that uses
    Object's [[Get]] method to access the 'get' property (8.10.5 step
    7.a)
---*/

var obj = {};

var regObj = new RegExp();

regObj.get = function() {
  return "regExpGetProperty";
};

Object.defineProperty(obj, "property", regObj);

assert.sameValue(obj.property, "regExpGetProperty", 'obj.property');
