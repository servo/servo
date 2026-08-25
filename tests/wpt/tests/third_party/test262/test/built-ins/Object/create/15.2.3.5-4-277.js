// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-277
description: >
    Object.create - 'set' property of one property in 'Properties' is
    own accessor property without a get function, which overrides an
    inherited accessor property (8.10.5 step 8.a)
---*/

var proto = {};
Object.defineProperty(proto, "set", {
  get: function() {
    return function() {};
  }
});

var ConstructFun = function() {};
ConstructFun.prototype = proto;
var child = new ConstructFun();
Object.defineProperty(child, "set", {
  set: function() {}
});

var newObj = Object.create({}, {
  prop: child
});

var desc = Object.getOwnPropertyDescriptor(newObj, "prop");

assert(newObj.hasOwnProperty("prop"), 'newObj.hasOwnProperty("prop") !== true');
assert.sameValue(typeof desc.set, "undefined", 'typeof desc.set');
