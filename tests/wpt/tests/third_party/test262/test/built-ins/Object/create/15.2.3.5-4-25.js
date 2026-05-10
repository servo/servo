// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-25
description: >
    Object.create - own enumerable accessor property that overrides an
    enumerable inherited accessor property in 'Properties' is defined
    in 'obj' (15.2.3.7 step 5.a)
---*/

var proto = {};
Object.defineProperty(proto, "prop", {
  get: function() {
    return {
      value: 9
    };
  },
  enumerable: true
});

var ConstructFun = function() {};
ConstructFun.prototype = proto;

var child = new ConstructFun();
Object.defineProperty(child, "prop", {
  get: function() {
    return {
      value: 12
    };
  },
  enumerable: true
});
var newObj = Object.create({}, child);

assert(newObj.hasOwnProperty("prop"), 'newObj.hasOwnProperty("prop") !== true');
assert.sameValue(newObj.prop, 12, 'newObj.prop');
