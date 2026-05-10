// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-27
description: >
    Object.defineProperty - 'name' is an inherited accessor property
    (8.12.9 step 1)
---*/

var proto = {};
Object.defineProperty(proto, "property", {
  get: function() {
    return 11;
  },
  configurable: false
});

var ConstructFun = function() {};
ConstructFun.prototype = proto;
var obj = new ConstructFun();

Object.defineProperty(obj, "property", {
  get: function() {
    return 12;
  },
  configurable: true
});

assert(obj.hasOwnProperty("property"), 'obj.hasOwnProperty("property") !== true');
assert.sameValue(obj.property, 12, 'obj.property');
