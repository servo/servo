// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-232
description: >
    Object.defineProperty - value of 'get' property in 'Attributes' is
    a function (8.10.5 step 7.b)
---*/

var obj = {};

Object.defineProperty(obj, "property", {
  get: function() {
    return "getFunction";
  }
});

assert(obj.hasOwnProperty("property"), 'obj.hasOwnProperty("property") !== true');
assert.sameValue(obj.property, "getFunction", 'obj.property');
