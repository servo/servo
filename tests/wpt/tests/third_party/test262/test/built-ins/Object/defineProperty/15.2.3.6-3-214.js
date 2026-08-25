// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-214
description: >
    Object.defineProperty - 'get' property in 'Attributes' is own
    accessor property that overrides an inherited accessor property
    (8.10.5 step 7.a)
---*/

var obj = {};
var proto = {};
Object.defineProperty(proto, "get", {
  get: function() {
    return function() {
      return "inheritedAccessorProperty";
    };
  }
});

var ConstructFun = function() {};
ConstructFun.prototype = proto;

var child = new ConstructFun();
Object.defineProperty(child, "get", {
  get: function() {
    return function() {
      return "ownAccessorProperty";
    };
  }
});

Object.defineProperty(obj, "property", child);

assert.sameValue(obj.property, "ownAccessorProperty", 'obj.property');
