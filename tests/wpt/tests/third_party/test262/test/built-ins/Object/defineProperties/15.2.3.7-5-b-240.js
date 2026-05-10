// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-5-b-240
description: >
    Object.defineProperties - 'descObj' is an Array object which
    implements its own [[Get]] method to get 'set' property (8.10.5
    step 8.a)
---*/

var data = "data";
var arr = [];
var setFun = function(value) {
  data = value;
};
arr.prop = {
  set: setFun
};

var obj = {};
Object.defineProperties(obj, arr);
obj.prop = "arrData";

assert(obj.hasOwnProperty("prop"), 'obj.hasOwnProperty("prop") !== true');
assert.sameValue(data, "arrData", 'data');
