// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-27
description: >
    Object.create - own enumerable accessor property in 'Properties'
    without a get function that overrides an enumerable inherited
    accessor property in 'Properties' is defined in 'obj' (15.2.3.7
    step 5.a)
---*/

var proto = {};
Object.defineProperty(proto, "prop", {
  get: function() {
    return {};
  },
  enumerable: true
});

var ConstructFun = function() {};
ConstructFun.prototype = proto;

var child = new ConstructFun();
Object.defineProperty(child, "prop", {
  set: function() {},
  enumerable: true
});
assert.throws(TypeError, function() {
  Object.create({}, child);
});
