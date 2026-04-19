// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.14-5-12
description: >
    Object.keys - own enumerable indexed accessor property of dense
    array 'O' is defined in returned array
---*/

var propertyFound = false;

var obj = [2, 3, 4, 5];

Object.defineProperty(obj, "prop", {
  get: function() {
    return 6;
  },
  enumerable: true,
  configurable: true
});

var arr = Object.keys(obj);

for (var p in arr) {
  if (arr.hasOwnProperty(p)) {
    if (arr[p] === "prop") {
      propertyFound = true;
      break;
    }
  }
}

assert(propertyFound, 'Property not found');
