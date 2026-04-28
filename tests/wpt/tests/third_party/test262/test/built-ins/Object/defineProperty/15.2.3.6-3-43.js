// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-43
description: >
    Object.defineProperty - 'Attributes' is an Arguments object which
    implements its own [[Get]] method to access the 'enumerable'
    property (8.10.5 step 3.a)
---*/

var obj = {};
var accessed = false;

var argObj = (function() {
  return arguments;
})();
argObj.enumerable = true;

Object.defineProperty(obj, "property", argObj);

for (var prop in obj) {
  if (prop === "property") {
    accessed = true;
  }
}

assert(accessed, 'accessed !== true');
