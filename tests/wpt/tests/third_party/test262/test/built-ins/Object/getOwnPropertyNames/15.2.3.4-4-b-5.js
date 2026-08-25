// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.4-4-b-5
description: >
    Object.getOwnPropertyNames - elements of the returned array are
    enumerable
---*/

var propertyFound = false;

var obj = {
  "a": "a"
};

var result = Object.getOwnPropertyNames(obj);

for (var p in result) {
  if (result[p] === "a") {
    propertyFound = true;
    break;
  }
}

assert(propertyFound, 'Property not found');
