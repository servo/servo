// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-191
description: >
    Object.defineProperty - 'writable' property in 'Attributes' is an
    Array object (8.10.5 step 6.b)
---*/

var obj = {};

Object.defineProperty(obj, "property", {
  writable: [1, 2, 3]
});

var beforeWrite = obj.hasOwnProperty("property");

obj.property = "isWritable";

var afterWrite = (obj.property === "isWritable");

assert.sameValue(beforeWrite, true, 'beforeWrite');
assert.sameValue(afterWrite, true, 'afterWrite');
