// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-226
description: >
    Object.defineProperty - 'Attributes' is the JSON object that uses
    Object's [[Get]] method to access the 'get' property (8.10.5 step
    7.a)
---*/

var obj = {};

JSON.get = function() {
  return "jsonGetProperty";
};

Object.defineProperty(obj, "property", JSON);

assert.sameValue(obj.property, "jsonGetProperty", 'obj.property');
