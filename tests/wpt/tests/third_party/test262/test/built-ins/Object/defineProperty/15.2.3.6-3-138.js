// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-138
description: >
    Object.defineProperty - 'value' property in 'Attributes' is an
    inherited accessor property without a get function  (8.10.5 step
    5.a)
---*/

var obj = {
  property: 120
};

var proto = {};
Object.defineProperty(proto, "value", {
  set: function() {}
});

var ConstructFun = function() {};
ConstructFun.prototype = proto;

var child = new ConstructFun();

Object.defineProperty(obj, "property", child);

assert(obj.hasOwnProperty("property"), 'obj.hasOwnProperty("property") !== true');
assert.sameValue(typeof(obj.property), "undefined", 'typeof (obj.property)');
