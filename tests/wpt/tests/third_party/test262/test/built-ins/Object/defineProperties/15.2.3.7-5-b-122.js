// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-5-b-122
description: >
    Object.defineProperties - 'value' property of 'descObj' is own
    accessor property without a get function (8.10.5 step 5.a)
---*/

var obj = {};

var descObj = {};

Object.defineProperty(descObj, "value", {
  set: function() {}
});

Object.defineProperties(obj, {
  property: descObj
});

assert(obj.hasOwnProperty("property"), 'obj.hasOwnProperty("property") !== true');
assert.sameValue(typeof(obj.property), "undefined", 'typeof (obj.property)');
