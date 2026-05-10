// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-24
description: >
    Object.defineProperty - 'name' is own data property that overrides
    an inherited data property (8.12.9 step 1)
---*/

var proto = {};
Object.defineProperty(proto, "foo", {
  value: 12,
  configurable: true
});

var ConstructFun = function() {};
ConstructFun.prototype = proto;
var obj = new ConstructFun();
Object.defineProperty(obj, "foo", {
  value: 11,
  configurable: false
});
assert.throws(TypeError, function() {
  Object.defineProperty(obj, "foo", {
    configurable: true
  });
});
assert.sameValue(obj.foo, 11, 'obj.foo');
