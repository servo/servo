// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-37
description: >
    Object.defineProperty - 'Attributes' is a Number object that uses
    Object's [[Get]] method to access the 'enumerable' property
    (8.10.5 step 3.a)
---*/

var obj = {};
var accessed = false;

var numObj = new Number(-2);
numObj.enumerable = true;

Object.defineProperty(obj, "property", numObj);

for (var prop in obj) {
  if (prop === "property") {
    accessed = true;
  }
}

assert(accessed, 'accessed !== true');
