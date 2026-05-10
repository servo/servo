// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-246
description: >
    Object.defineProperty - 'set' property in 'Attributes' is own
    accessor property(without a get function) that overrides an
    inherited accessor property (8.10.5 step 8.a)
includes: [propertyHelper.js]
---*/

var obj = {};
var proto = {};
var data = "data";
Object.defineProperty(proto, "set", {
  get: function() {
    return function(value) {
      data = value;
    };
  }
});

var ConstructFun = function() {};
ConstructFun.prototype = proto;

var child = new ConstructFun();
Object.defineProperty(child, "set", {
  set: function() {}
});

Object.defineProperty(obj, "property", child);

verifyNotWritable(obj, "property");

assert.sameValue(typeof obj.property, "undefined");
assert.sameValue(data, "data");

assert(obj.hasOwnProperty("property"));
