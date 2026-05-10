// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-5-b-231
description: >
    Object.defineProperties - 'set' property of 'descObj' is own data
    property that overrides an inherited accessor property (8.10.5
    step 8.a)
---*/

var data1 = "data";
var data2 = "data";
var fun = function(value) {
  data2 = value;
};
var proto = {};
Object.defineProperty(proto, "set", {
  get: function() {
    return fun;
  },
  set: function(value) {
    fun = value;
  }
});

var Con = function() {};
Con.prototype = proto;

var child = new Con();
child.set = function(value) {
  data1 = value;
};

var obj = {};

Object.defineProperties(obj, {
  prop: child
});

obj.prop = "overrideData";

assert(obj.hasOwnProperty("prop"), 'obj.hasOwnProperty("prop") !== true');
assert.sameValue(data1, "overrideData", 'data1');
assert.sameValue(data2, "data", 'data2');
