// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-5-b-232
description: >
    Object.defineProperties - 'set' property of 'descObj' is own
    accessor property (8.10.5 step 8.a)
---*/

var data = "data";
var setFun = function(value) {
  data = value;
};
var descObj = {};
Object.defineProperty(descObj, "set", {
  get: function() {
    return setFun;
  }
});

var obj = {};

Object.defineProperties(obj, {
  prop: descObj
});

obj.prop = "overrideData";

assert(obj.hasOwnProperty("prop"), 'obj.hasOwnProperty("prop") !== true');
assert.sameValue(data, "overrideData", 'data');
