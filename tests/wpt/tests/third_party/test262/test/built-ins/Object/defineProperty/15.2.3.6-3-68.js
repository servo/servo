// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-68
description: >
    Object.defineProperty - value of 'enumerable' property in
    'Attributes' is an Arguments Object (8.10.5 step 3.b)
---*/

var obj = {};
var accessed = false;
var argObj = (function() {
  return arguments;
})(0, 1, 2);

Object.defineProperty(obj, "property", {
  enumerable: argObj
});

for (var prop in obj) {
  if (prop === "property") {
    accessed = true;
  }
}

assert(accessed, 'accessed !== true');
