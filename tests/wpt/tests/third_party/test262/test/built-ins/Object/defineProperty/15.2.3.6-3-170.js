// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-170
description: >
    Object.defineProperty - 'Attributes' is the Math object that uses
    Object's [[Get]] method to access the 'writable' property  (8.10.5
    step 6.a)
---*/

var obj = {};

Math.writable = true;

Object.defineProperty(obj, "property", Math);

var beforeWrite = obj.hasOwnProperty("property");

obj.property = "isWritable";

var afterWrite = (obj.property === "isWritable");

assert.sameValue(beforeWrite, true, 'beforeWrite');
assert.sameValue(afterWrite, true, 'afterWrite');
