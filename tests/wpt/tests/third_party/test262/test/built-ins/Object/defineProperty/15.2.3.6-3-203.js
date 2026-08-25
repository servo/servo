// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-203
description: >
    Object.defineProperty - 'writable' property in 'Attributes' is
    treated as true when it is a string(value is 'false') (8.10.5 step
    6.b)
---*/

var obj = {};

Object.defineProperty(obj, "property", {
  writable: "false"
});

var beforeWrite = obj.hasOwnProperty("property");

obj.property = "isWritable";

var afterWrite = (obj.property === "isWritable");

assert.sameValue(beforeWrite, true, 'beforeWrite');
assert.sameValue(afterWrite, true, 'afterWrite');
