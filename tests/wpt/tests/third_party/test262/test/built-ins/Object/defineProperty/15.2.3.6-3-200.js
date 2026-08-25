// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-200
description: >
    Object.defineProperty - 'writable' property in 'Attributes' is the
    Argument object (8.10.5 step 6.b)
---*/

var obj = {};

var argObj = (function() {
  return arguments;
})(1, true, "a");

Object.defineProperty(obj, "property", {
  writable: argObj
});

var beforeWrite = obj.hasOwnProperty("property");

obj.property = "isWritable";

var afterWrite = (obj.property === "isWritable");

assert.sameValue(beforeWrite, true, 'beforeWrite');
assert.sameValue(afterWrite, true, 'afterWrite');
