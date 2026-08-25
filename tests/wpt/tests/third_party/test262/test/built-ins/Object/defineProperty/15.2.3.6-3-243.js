// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-243
description: >
    Object.defineProperty - 'set' property in 'Attributes' is own
    accessor property that overrides an inherited data property
    (8.10.5 step 8.a)
---*/

var obj = {};
var data1 = "data";
var data2 = "data";

var proto = {
  set: function(value) {
    data1 = value;
  }
};

var ConstructFun = function() {};
ConstructFun.prototype = proto;

var child = new ConstructFun();
Object.defineProperty(child, "set", {
  get: function() {
    return function(value) {
      data2 = value;
    };
  }
});

Object.defineProperty(obj, "property", child);
obj.property = "ownAccessorProperty";

assert(obj.hasOwnProperty("property"), 'obj.hasOwnProperty("property") !== true');
assert.sameValue(data1, "data", 'data1');
assert.sameValue(data2, "ownAccessorProperty", 'data2');
