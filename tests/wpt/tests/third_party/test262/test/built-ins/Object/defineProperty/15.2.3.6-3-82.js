// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-82
description: >
    Object.defineProperty - 'configurable' property in 'Attributes' is
    own accessor property that overrides an inherited accessor
    property (8.10.5 step 4.a)
---*/

var obj = {};

var proto = {};
Object.defineProperty(proto, "configurable", {
  get: function() {
    return false;
  }
});

var ConstructFun = function() {};
ConstructFun.prototype = proto;

var child = new ConstructFun();
Object.defineProperty(child, "configurable", {
  get: function() {
    return true;
  }
});

Object.defineProperty(obj, "property", child);

var beforeDeleted = obj.hasOwnProperty("property");

delete obj.property;

var afterDeleted = obj.hasOwnProperty("property");

assert.sameValue(beforeDeleted, true, 'beforeDeleted');
assert.sameValue(afterDeleted, false, 'afterDeleted');
