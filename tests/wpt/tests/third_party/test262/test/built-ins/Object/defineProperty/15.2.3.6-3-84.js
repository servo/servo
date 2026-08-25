// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-84
description: >
    Object.defineProperty - 'configurable' property in 'Attributes' is
    own accessor property(without a get function) that overrides an
    inherited accessor property (8.10.5 step 4.a)
includes: [propertyHelper.js]
---*/

var obj = {};

var proto = {};
Object.defineProperty(proto, "configurable", {
  get: function() {
    return true;
  }
});

var ConstructFun = function() {};
ConstructFun.prototype = proto;

var child = new ConstructFun();
Object.defineProperty(child, "configurable", {
  set: function() {}
});

Object.defineProperty(obj, "property", child);

verifyProperty(obj, "property", {
  configurable: false,
});
