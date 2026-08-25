// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-43-1
description: >
    Object.defineProperty - 'Attributes' is an Arguments object which
    implements its own [[Get]] method to access the 'enumerable'
    property of prototype object (8.10.5 step 3.a)
---*/

var obj = {};
var accessed = false;

Object.prototype.enumerable = true;
var argObj = (function() {
  return arguments;
})();

Object.defineProperty(obj, "property", argObj);

for (var prop in obj) {
  if (prop === "property") {
    accessed = true;
  }
}

assert(accessed, 'accessed !== true');
