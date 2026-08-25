// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-21
description: >
    Object.defineProperty - 'enumerable' property in 'Attributes' is
    not present (8.10.5 step 3)
---*/

var obj = {};

var attr = {};
var accessed = false;
Object.defineProperty(obj, "property", attr);

for (var prop in obj) {
  if (prop === "property") {
    accessed = true;
  }
}

assert.sameValue(accessed, false, 'accessed');
