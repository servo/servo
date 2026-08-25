// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-46
description: >
    Object.defineProperty - 'name' is defined as data property if
    'name' property doesn't exist in 'O' and 'desc' is generic
    descriptor (8.12.9 step 4.a)
---*/

var obj = {};

Object.defineProperty(obj, "property", {
  enumerable: true
});

var isEnumerable = false;
for (var item in obj) {
  if (obj.hasOwnProperty(item) && item === "property") {
    isEnumerable = true;
  }
}

assert(obj.hasOwnProperty("property"), 'obj.hasOwnProperty("property") !== true');
assert(isEnumerable, 'isEnumerable !== true');
