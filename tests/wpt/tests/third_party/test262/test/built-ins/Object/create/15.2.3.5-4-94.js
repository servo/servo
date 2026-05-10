// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-94
description: >
    Object.create - 'enumerable' property of one property in
    'Properties' is an Arguments object (8.10.5 step 3.b)
---*/

var accessed = false;
var argObj = (function() {
  return arguments;
})();

var newObj = Object.create({}, {
  prop: {
    enumerable: argObj
  }
});
for (var property in newObj) {
  if (property === "prop") {
    accessed = true;
  }
}

assert(accessed, 'accessed !== true');
