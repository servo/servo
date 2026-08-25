// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-257
description: >
    Object.defineProperty - 'Attributes' is an Error object that uses
    Object's [[Get]] method to access the 'set' property (8.10.5 step
    8.a)
---*/

var obj = {};
var data = "data";
var errObj = new Error();

errObj.set = function(value) {
  data = value;
};

Object.defineProperty(obj, "property", errObj);
obj.property = "overrideData";

assert(obj.hasOwnProperty("property"), 'obj.hasOwnProperty("property") !== true');
assert.sameValue(data, "overrideData", 'data');
