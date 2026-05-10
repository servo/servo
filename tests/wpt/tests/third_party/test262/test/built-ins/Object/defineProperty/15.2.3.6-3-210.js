// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-210
description: >
    Object.defineProperty - 'get' property in 'Attributes' is own data
    property that overrides an inherited accessor property (8.10.5
    step 7.a)
---*/

var obj = {};
var proto = {};
var fun = function() {
  return "inheritedAccessorProperty";
};
Object.defineProperty(proto, "get", {
  get: function() {
    return fun;
  }
});

var ConstructFun = function() {};
ConstructFun.prototype = proto;

var child = new ConstructFun();
Object.defineProperty(child, "get", {
  value: function() {
    return "ownDataProperty";
  }
});

Object.defineProperty(obj, "property", child);

assert.sameValue(obj.property, "ownDataProperty", 'obj.property');
