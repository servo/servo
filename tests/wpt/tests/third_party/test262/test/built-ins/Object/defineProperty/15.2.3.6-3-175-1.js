// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-175-1
description: >
    Object.defineProperty - 'Attributes' is an Arguments object which
    implements its own [[Get]] method to access the 'writable'
    property of prototype object (8.10.5 step 6.b)
---*/

var obj = {};

Object.prototype.writable = true;

var argObj = (function() {
  return arguments;
})();

Object.defineProperty(obj, "property", argObj);

var beforeWrite = obj.hasOwnProperty("property");

obj.property = "isWritable";

var afterWrite = (obj.property === "isWritable");

assert.sameValue(beforeWrite, true, 'beforeWrite');
assert.sameValue(afterWrite, true, 'afterWrite');
