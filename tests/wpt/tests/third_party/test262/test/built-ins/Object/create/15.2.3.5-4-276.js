// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-276
description: >
    Object.create - 'set' property of one property in 'Properties' is
    own accessor property without a get function (8.10.5 step 8.a)
---*/

var descObj = {};
Object.defineProperty(descObj, "set", {
  set: function() {}
});

var newObj = Object.create({}, {
  prop: descObj
});

var hasProperty = newObj.hasOwnProperty("prop");

var desc = Object.getOwnPropertyDescriptor(newObj, "prop");

assert(hasProperty, 'hasProperty !== true');
assert.sameValue(typeof desc.set, "undefined", 'typeof desc.set');
