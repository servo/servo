// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-5-b-24
description: >
    Object.defineProperties - 'descObj' is the Math object which
    implements its own [[Get]] method to get 'enumerable' property
    (8.10.5 step 3.a)
---*/

var obj = {};
var accessed = false;

Math.enumerable = true;

Object.defineProperties(obj, {
  prop: Math
});
for (var property in obj) {
  if (property === "prop") {
    accessed = true;
  }
}

assert(accessed, 'accessed !== true');
