// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-240
description: >
    Object.defineProperty - 'set' property in 'Attributes' is own data
    property that overrides an inherited accessor property (8.10.5
    step 8.a)
---*/

var obj = {};
var proto = {};
var data1 = "data";
var data2 = "data";

Object.defineProperty(proto, "set", {
  get: function() {
    return function(value) {
      data1 = value;
    };
  }
});

var ConstructFun = function() {};
ConstructFun.prototype = proto;

var child = new ConstructFun();
Object.defineProperty(child, "set", {
  value: function(value) {
    data2 = value;
  }
});

Object.defineProperty(obj, "property", child);

obj.property = "overrideData";

assert(obj.hasOwnProperty("property"), 'obj.hasOwnProperty("property") !== true');
assert.sameValue(data1, "data", 'data1');
assert.sameValue(data2, "overrideData", 'data2');
