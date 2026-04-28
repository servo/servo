// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-26
description: >
    Object.defineProperty - 'enumerable' property in 'Attributes' is
    own accessor property (8.10.5 step 3.a)
---*/

var obj = {};
var accessed = false;

var attr = {};
Object.defineProperty(attr, "enumerable", {
  get: function() {
    return true;
  }
});

Object.defineProperty(obj, "property", attr);

for (var prop in obj) {
  if (prop === "property") {
    accessed = true;
  }
}

assert(accessed, 'accessed !== true');
