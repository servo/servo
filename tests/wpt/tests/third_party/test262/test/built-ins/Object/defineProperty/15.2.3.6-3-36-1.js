// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-36-1
description: >
    Object.defineProperty - 'Attributes' is a Boolean object that uses
    Object's [[Get]] method to access the 'enumerable' property of
    prototype object (8.10.5 step 3.a)
---*/

var obj = {};
var accessed = false;

Boolean.prototype.enumerable = true;
var boolObj = new Boolean(true);

Object.defineProperty(obj, "property", boolObj);

for (var prop in obj) {
  if (prop === "property") {
    accessed = true;
  }
}

assert(accessed, 'accessed !== true');
