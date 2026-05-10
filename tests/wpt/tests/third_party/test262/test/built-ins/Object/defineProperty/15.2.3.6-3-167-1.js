// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-167-1
description: >
    Object.defineProperty - 'Attributes' is a String object that uses
    Object's [[Get]] method to access the 'writable' property of
    prototype object (8.10.5 step 6.b)
---*/

var obj = {};

String.prototype.writable = true;
var strObj = new String("abc");

Object.defineProperty(obj, "property", strObj);

var beforeWrite = obj.hasOwnProperty("property");

obj.property = "isWritable";

var afterWrite = (obj.property === "isWritable");

assert.sameValue(beforeWrite, true, 'beforeWrite');
assert.sameValue(afterWrite, true, 'afterWrite');
