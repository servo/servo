// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-248-1
description: >
    Object.defineProperty - 'Attributes' is a Function object which
    implements its own [[Get]] method to access the 'set' property of
    prototype object (8.10.5 step 8.a)
---*/

var obj = {};
var data = "data";

Function.prototype.set = function(value) {
  data = value;
};
var funObj = function() {};

Object.defineProperty(obj, "property", funObj);
obj.property = "overrideData";

assert(obj.hasOwnProperty("property"), 'obj.hasOwnProperty("property") !== true');
assert.sameValue(data, "overrideData", 'data');
