// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-46
description: >
    Object.create - 'enumerable' property of one property in
    'Properties' is true (8.10.5 step 3)
---*/

var accessed = false;
var newObj = Object.create({}, {
  prop: {
    enumerable: true
  }
});
for (var property in newObj) {
  if (property === "prop") {
    accessed = true;
  }
}

assert(accessed, 'accessed !== true');
