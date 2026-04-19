// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-40-1
description: >
    Object.defineProperty - 'Attributes' is an RegExp object that uses
    Object's [[Get]] method to access the 'enumerable' property of
    prototype object (8.10.5 step 3.a)
---*/

var obj = {};
var accessed = false;

RegExp.prototype.enumerable = true;
var regObj = new RegExp();

Object.defineProperty(obj, "property", regObj);

for (var prop in obj) {
  if (prop === "property") {
    accessed = true;
  }
}

assert(accessed, 'accessed !== true');
