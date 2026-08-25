// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-5-b-229
description: >
    Object.defineProperties - 'set' property of 'descObj' is inherited
    data property (8.10.5 step 8.a)
---*/

var data = "data";
var proto = {
  set: function(value) {
    data = value;
  }
};

var Con = function() {};
Con.prototype = proto;

var child = new Con();
var obj = {};

Object.defineProperties(obj, {
  prop: child
});

obj.prop = "overrideData";

assert(obj.hasOwnProperty("prop"), 'obj.hasOwnProperty("prop") !== true');
assert.sameValue(data, "overrideData", 'data');
