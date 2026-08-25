// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-208
description: >
    Object.defineProperty - 'get' property in 'Attributes' is an
    inherited data property (8.10.5 step 7.a)
---*/

var obj = {};
var proto = {
  get: function() {
    return "inheritedDataProperty";
  }
};

var ConstructFun = function() {};
ConstructFun.prototype = proto;

var child = new ConstructFun();

Object.defineProperty(obj, "property", child);

assert.sameValue(obj.property, "inheritedDataProperty", 'obj.property');
