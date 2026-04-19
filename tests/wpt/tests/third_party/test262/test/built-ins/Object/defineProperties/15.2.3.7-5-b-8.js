// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-5-b-8
description: >
    Object.defineProperties - 'enumerable' property of 'descObj' is
    own data property (8.10.5 step 3.a)
---*/

var obj = {};
var accessed = false;

var descObj = {
  enumerable: true
};

Object.defineProperties(obj, {
  prop: descObj
});

for (var property in obj) {
  if (property === "prop") {
    accessed = true;
  }
}

assert(accessed, 'accessed !== true');
