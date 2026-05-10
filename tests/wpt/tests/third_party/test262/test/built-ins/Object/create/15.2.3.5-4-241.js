// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-241
description: >
    Object.create - 'get' property of one property in 'Properties' is
    own accessor property without a get function (8.10.5 step 7.a)
---*/

var descObj = {};

Object.defineProperty(descObj, "get", {
  set: function() {}
});

var newObj = Object.create({}, {
  prop: descObj
});

assert(newObj.hasOwnProperty("prop"), 'newObj.hasOwnProperty("prop") !== true');
assert.sameValue(typeof(newObj.prop), "undefined", 'typeof (newObj.prop)');
