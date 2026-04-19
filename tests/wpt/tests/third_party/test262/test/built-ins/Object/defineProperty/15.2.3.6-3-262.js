// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-262
description: >
    Object.defineProperty - value of 'set' property in 'Attributes' is
    a function (8.10.5 step 8.b)
---*/

var obj = {};
var data = "data";

Object.defineProperty(obj, "property", {
  set: function(value) {
    data = value;
  }
});
obj.property = "overrideData";

assert(obj.hasOwnProperty("property"), 'obj.hasOwnProperty("property") !== true');
assert.sameValue(data, "overrideData", 'data');
