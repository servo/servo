// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-236
description: >
    Object.create - 'get' property of one property in 'Properties' is
    own data property that overrides an inherited accessor property
    (8.10.5 step 7.a)
---*/

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
var descObj = new ConstructFun();

Object.defineProperty(descObj, "get", {
  value: function() {
    return "ownDataProperty";
  }
});

var newObj = Object.create({}, {
  prop: descObj
});

assert.sameValue(newObj.prop, "ownDataProperty", 'newObj.prop');
