// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-201
description: >
    Object.create - one property in 'Properties' is an Arguments
    object which implements its own [[Get]] method to access the
    'writable' property (8.10.5 step 6.a)
---*/

var argObj = (function() {
  return arguments;
})();

argObj.writable = true;

var newObj = Object.create({}, {
  prop: argObj
});

var beforeWrite = (newObj.hasOwnProperty("prop") && typeof(newObj.prop) === "undefined");

newObj.prop = "isWritable";

var afterWrite = (newObj.prop === "isWritable");

assert.sameValue(beforeWrite, true, 'beforeWrite');
assert.sameValue(afterWrite, true, 'afterWrite');
