// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-5-b-258
description: >
    Object.defineProperties - value of 'set' property of 'descObj' is
    a function (8.10.5 step 8.b)
---*/

var data = "data";
var setFun = function(value) {
  data = value;
};
var obj = {};


Object.defineProperties(obj, {
  prop: {
    set: setFun
  }
});
obj.prop = "funData";

assert(obj.hasOwnProperty("prop"), 'obj.hasOwnProperty("prop") !== true');
assert.sameValue(data, "funData", 'data');
