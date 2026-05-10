// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-69
description: >
    Object.create - one property in 'Properties' is an Arguments
    object which implements its own [[Get]] method to access the
    'enumerable' property (8.10.5 step 3.a)
---*/

var accessed = false;
var argObj = (function() {
  return arguments;
})();

argObj.enumerable = true;

var newObj = Object.create({}, {
  prop: argObj
});
for (var property in newObj) {
  if (property === "prop") {
    accessed = true;
  }
}

assert(accessed, 'accessed !== true');
