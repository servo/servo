// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-159
description: >
    Object.create - 'value' property of one property in 'Properties'
    is an inherited accessor property (8.10.5 step 5.a)
---*/

var proto = {};

Object.defineProperty(proto, "value", {
  get: function() {
    return "inheritedAccessorProperty";
  }
});

var ConstructFun = function() {};
ConstructFun.prototype = proto;

var descObj = new ConstructFun();

var newObj = Object.create({}, {
  prop: descObj
});

assert.sameValue(newObj.prop, "inheritedAccessorProperty", 'newObj.prop');
