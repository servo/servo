// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-5-b-110
description: >
    Object.defineProperties - value of 'configurable' property of
    'descObj' is a string (value is 'false') which is treated as true
    value (8.10.5 step 4.b)
---*/

var obj = {};

Object.defineProperties(obj, {
  property: {
    configurable: "false"
  }
});
var preCheck = obj.hasOwnProperty("property");
delete obj.property;

assert(preCheck, 'preCheck !== true');
assert.sameValue(obj.hasOwnProperty("property"), false, 'obj.hasOwnProperty("property")');
