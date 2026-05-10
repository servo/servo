// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-20
description: >
    Object.defineProperty - 'enumerable' property in 'Attributes' is
    present (8.10.5 step 3)
---*/

var obj = {};

var accessed = false;
Object.defineProperty(obj, "property", {
  enumerable: false
});

for (var prop in obj) {
  if (prop === "property") {
    accessed = true;
  }
}

assert.sameValue(accessed, false, 'accessed');
assert(obj.hasOwnProperty("property"), 'obj.hasOwnProperty("property") !== true');
