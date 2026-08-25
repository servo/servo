// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-96
description: >
    Object.create - 'enumerable' property of one property in
    'Properties' is the global object (8.10.5 step 3.b)
---*/

var accessed = false;

var newObj = Object.create({}, {
  prop: {
    enumerable: this
  }
});
for (var property in newObj) {
  if (property === "prop") {
    accessed = true;
  }
}

assert(accessed, 'accessed !== true');
