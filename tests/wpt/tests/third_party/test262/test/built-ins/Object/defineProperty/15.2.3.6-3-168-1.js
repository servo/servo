// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-168-1
description: >
    Object.defineProperty - 'Attributes' is a Boolean object that uses
    Object's [[Get]] method to access the 'writable' property of
    prototype object (8.10.5 step 6.b)
---*/

var obj = {};

Boolean.prototype.writable = true;
var boolObj = new Boolean(true);

Object.defineProperty(obj, "property", boolObj);

var beforeWrite = obj.hasOwnProperty("property");

obj.property = "isWritable";

var afterWrite = (obj.property === "isWritable");

assert.sameValue(beforeWrite, true, 'beforeWrite');
assert.sameValue(afterWrite, true, 'afterWrite');
