// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.14-5-14
description: >
    Object.keys - own enumerable indexed accessor property of sparse
    array 'O' is defined in returned array
---*/

var propertyFound = false;

var obj = [1, , 3, , 5];

Object.defineProperty(obj, "10000", {
  get: function() {
    return "ElementWithLargeIndex";
  },
  enumerable: true,
  configurable: true
});

var arr = Object.keys(obj);

for (var p in arr) {
  if (arr[p] === "10000") {
    propertyFound = true;
    break;
  }
}

assert(propertyFound, 'Property not found');
